import AppKit
import MakoCore
import WebKit

/// The frame the page sits in. Keys the page and its text fields leave alone reach it,
/// because WebKit passes unhandled keys up the responder chain, and so does every resize.
final class PageContainer: NSView {
    var onUnhandledKey: (NSEvent) -> Bool = { _ in false }
    var onResize: () -> Void = {}

    override func keyDown(with event: NSEvent) {
        if !onUnhandledKey(event) { super.keyDown(with: event) }
    }

    override func setFrameSize(_ size: NSSize) {
        super.setFrameSize(size)
        onResize()
    }
}

/// Link hints, as in Vimium: a short label on every link and control in view, and typing a
/// label clicks what it marks. Beneath mode labels only web links and opens the chosen one
/// beneath the page without leaving it, like a middle-click. The core hands out the labels;
/// the page script only reports where things are.
@MainActor
final class LinkHints {
    private struct Target {
        let label: String
        /// Where the label goes and where the click lands, in CSS pixels from the viewport's top-left.
        let corner: CGPoint
        let point: CGPoint
        let link: URL?
        let name: String
    }

    private let overlay = HintOverlay()
    private weak var page: PageView?
    private var targets: [Target] = []
    private var typed = ""
    private var beneath = false
    private var color: Int?
    private var monitor: Any?
    /// Bumped on every start and cancel, so a late page reply never shows stale labels.
    private var generation = 0

    /// Visible links and controls: their top-left, a point on them that is not covered by
    /// anything else, the web address a link goes to, and a name for accessibility.
    private static let collect = """
        const pick = 'a[href], area[href], button, input:not([type=hidden]), select, textarea, summary, '
          + '[contenteditable=""], [contenteditable=true], [onclick], [tabindex]:not([tabindex="-1"]), '
          + '[role=button], [role=link], [role=checkbox], [role=radio], [role=switch], [role=tab], '
          + '[role=menuitem], [role=option], [role=treeitem]';
        const web = el => typeof el.href === 'string' && /^https?:/.test(el.href) && !el.hasAttribute('download');
        const found = [];
        for (const el of document.querySelectorAll(pick)) {
          if (el.disabled || (links && !web(el))) continue;
          for (const r of el.getClientRects()) {
            const left = Math.max(r.left, 0), top = Math.max(r.top, 0);
            const right = Math.min(r.right, innerWidth), bottom = Math.min(r.bottom, innerHeight);
            if (right - left < 2 || bottom - top < 2) continue;
            const x = (left + right) / 2, y = (top + bottom) / 2;
            if (document.elementFromPoint(x, y)?.closest(pick) !== el) continue;
            const name = (web(el) && el.href) || el.getAttribute('aria-label') || el.innerText?.trim().slice(0, 80)
              || el.getAttribute('placeholder') || el.tagName.toLowerCase();
            found.push({ left, top, x, y, link: web(el) ? el.href : null, name });
            break;
          }
        }
        return found;
        """

    /// Labels the page's links and controls, or beeps when there are none in view.
    func start(on page: PageView, in container: NSView, beneath: Bool, color: Int?) {
        cancel()
        guard page.magnification == 1 else { return NSSound.beep() }
        let generation = generation
        page.callAsyncJavaScript(Self.collect, arguments: ["links": beneath], in: nil, in: .defaultClient) { [weak self, weak page] result in
            MainActor.assumeIsolated {
                guard let self, let page, generation == self.generation else { return }
                let found = (try? result.get()) as? [[String: Any]] ?? []
                self.show(found, on: page, in: container, beneath: beneath, color: color)
            }
        }
    }

    func cancel() {
        generation += 1
        if let monitor { NSEvent.removeMonitor(monitor) }
        monitor = nil
        targets = []
        typed = ""
        overlay.removeFromSuperview()
    }

    private func show(_ found: [[String: Any]], on page: PageView, in container: NSView, beneath: Bool, color: Int?) {
        let labels = Self.labels(found.count)
        targets = zip(labels, found).compactMap { label, t in
            guard let left = t["left"] as? Double, let top = t["top"] as? Double,
                  let x = t["x"] as? Double, let y = t["y"] as? Double else { return nil }
            return Target(label: label, corner: CGPoint(x: left, y: top), point: CGPoint(x: x, y: y),
                          link: (t["link"] as? String).flatMap(URL.init(string:)), name: t["name"] as? String ?? "")
        }
        guard !targets.isEmpty else { return NSSound.beep() }
        self.page = page
        self.beneath = beneath
        self.color = color
        overlay.frame = container.bounds
        overlay.autoresizingMask = [.width, .height]
        container.addSubview(overlay)
        render()
        monitor = NSEvent.addLocalMonitorForEvents(matching: [.keyDown, .leftMouseDown, .rightMouseDown, .otherMouseDown, .scrollWheel]) { [weak self] event in
            MainActor.assumeIsolated { self?.passes(event) ?? true } ? event : nil
        }
    }

    private static func labels(_ count: Int) -> [String] {
        guard count > 0, let p = mako_hint_labels(UInt32(count)) else { return [] }
        defer { mako_free(p) }
        return String(cString: p).split(separator: "\n").map(String.init)
    }

    private func render() {
        guard let page else { return }
        let shown = targets.filter { $0.label.hasPrefix(typed) }
        overlay.show(shown.map { t in
            HintChip(label: t.label, typed: typed.count, name: t.name, color: color) { [weak self] in self?.follow(t) }
        }, at: shown.map { page.convert(page.viewPoint(css: $0.corner), to: overlay) })
    }

    /// Hint mode sees every key and click first. Letters pick, Delete takes one back, and
    /// Escape, a shortcut, a scroll or a click anywhere but a label ends it.
    private func passes(_ event: NSEvent) -> Bool {
        guard event.window === overlay.window else { cancel(); return true }
        switch event.type {
        case .keyDown:
            guard event.modifierFlags.isDisjoint(with: [.command, .control, .option]) else { cancel(); return true }
            switch event.keyCode {
            case 53: cancel()
            case 51: if !typed.isEmpty { typed.removeLast(); render() }
            default: type(event.charactersIgnoringModifiers?.lowercased() ?? "")
            }
            return false
        case .leftMouseDown where overlay.chip(at: event.locationInWindow):
            return true
        default:
            cancel()
            return true
        }
    }

    private func type(_ key: String) {
        let next = typed + key
        if let hit = targets.first(where: { $0.label == next }) { return follow(hit) }
        guard !key.isEmpty, targets.contains(where: { $0.label.hasPrefix(next) }) else { return NSSound.beep() }
        typed = next
        render()
    }

    private func follow(_ target: Target) {
        let page = page
        let beneath = beneath
        cancel()
        if beneath, let link = target.link { page?.onBackgroundLink(link) } else { page?.click(css: target.point) }
    }
}

/// Holds the chips over the page. Clicks anywhere else go through to the page.
private final class HintOverlay: NSView {
    override var isFlipped: Bool { true }

    func show(_ chips: [HintChip], at corners: [NSPoint]) {
        subviews.forEach { $0.removeFromSuperview() }
        for (chip, corner) in zip(chips, corners) {
            let size = chip.intrinsicContentSize
            chip.frame = NSRect(x: min(max(corner.x, 0), bounds.width - size.width),
                                y: min(max(corner.y, 0), bounds.height - size.height),
                                width: size.width, height: size.height)
            addSubview(chip)
        }
    }

    func chip(at locationInWindow: NSPoint) -> Bool {
        superview.map { hitTest($0.convert(locationInWindow, from: nil)) is HintChip } ?? false
    }

    override func hitTest(_ point: NSPoint) -> NSView? {
        let hit = super.hitTest(point)
        return hit === self ? nil : hit
    }
}

/// One label, in the active trail's colours. The letters typed so far fade.
private final class HintChip: NSView {
    private let text: NSAttributedString
    private let color: Int?
    private let pick: () -> Void
    private static let font = NSFont.monospacedSystemFont(ofSize: 11, weight: .bold)
    private static let pad = NSSize(width: 4, height: 1.5)

    init(label: String, typed: Int, name: String, color: Int?, pick: @escaping () -> Void) {
        self.color = color
        self.pick = pick
        let ink = color.map(NSColor.trail) ?? .ink
        let text = NSMutableAttributedString(string: label.uppercased(), attributes: [.font: Self.font, .foregroundColor: ink])
        text.addAttribute(.foregroundColor, value: ink.withAlphaComponent(0.35), range: NSRange(location: 0, length: typed))
        self.text = text
        super.init(frame: .zero)
        wantsLayer = true
        layer?.shadowOpacity = 1
        layer?.shadowRadius = 2
        layer?.shadowOffset = NSSize(width: 0, height: -1)
        setAccessibilityElement(true)
        setAccessibilityRole(.button)
        setAccessibilityIdentifier("mako.hint.\(label)")
        setAccessibilityLabel(name)
    }

    required init?(coder: NSCoder) { fatalError() }

    override var intrinsicContentSize: NSSize {
        let s = text.size()
        return NSSize(width: ceil(s.width) + 2 * Self.pad.width, height: ceil(s.height) + 2 * Self.pad.height)
    }

    override func draw(_: NSRect) {
        effectiveAppearance.performAsCurrentDrawingAppearance { layer?.shadowColor = NSColor.cardShadow.cgColor }
        let shape = NSBezierPath(roundedRect: bounds.insetBy(dx: 0.5, dy: 0.5), xRadius: 3, yRadius: 3)
        (color.map(NSColor.card) ?? .sidebarPaper).setFill()
        shape.fill()
        (color.map(NSColor.trail) ?? .ink).withAlphaComponent(0.6).setStroke()
        shape.lineWidth = 1
        shape.stroke()
        text.draw(at: NSPoint(x: Self.pad.width, y: Self.pad.height))
    }

    override var isFlipped: Bool { true }
    override func acceptsFirstMouse(for _: NSEvent?) -> Bool { true }
    override func mouseDown(with _: NSEvent) { pick() }
    override func accessibilityPerformPress() -> Bool { pick(); return true }
}
