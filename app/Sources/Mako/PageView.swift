import AppKit
import WebKit

/// A trail page's web view. A middle-click opens what it points at in the background:
/// a link's address directly, so single-page apps that route left clicks in place cannot
/// keep it; a download link (whose `download` attribute only WebKit's own activation honours)
/// and anything else get a real left click, and the navigation it starts within a
/// second opens in the background (see `claimBackground`). Stock WebKit gives non-link
/// elements only an auxclick, so script buttons would otherwise ignore the middle button.
final class PageView: WKWebView {
    /// A link under a middle-click, to open beneath this page without leaving it.
    var onBackgroundLink: (URL) -> Void = { _ in }
    private var backgroundUntil: Date?

    /// True once for the first navigation within a second of a replayed middle-click.
    func claimBackground() -> Bool {
        defer { backgroundUntil = nil }
        return backgroundUntil.map { $0 > .now } ?? false
    }

    /// Topmost link at a viewport point, read from Mako's own script world. Not a download link.
    private static let linkAt = """
        const a = document.elementFromPoint(x, y)?.closest('a[href], area[href]');
        return a && !a.hasAttribute('download') && /^https?:$/.test(new URL(a.href).protocol) ? a.href : null;
        """

    override func otherMouseDown(with event: NSEvent) {
        guard event.buttonNumber == 2 else { return super.otherMouseDown(with: event) }
    }

    override func otherMouseUp(with event: NSEvent) {
        guard event.buttonNumber == 2 else { return super.otherMouseUp(with: event) }
        let p = convert(event.locationInWindow, from: nil)
        let scale = pageZoom * magnification
        let point = ["x": p.x / scale, "y": (isFlipped ? p.y : bounds.height - p.y) / scale]
        callAsyncJavaScript(Self.linkAt, arguments: point, in: nil, in: .defaultClient) { [weak self] result in
            MainActor.assumeIsolated {
                guard let self else { return }
                // Pinch-zoomed pages shift the viewport, so the point may miss: then the click decides.
                if self.magnification == 1, case .success(let href as String) = result, let url = URL(string: href) {
                    self.onBackgroundLink(url)
                } else {
                    self.replayLeftClick(like: event)
                }
            }
        }
    }

    private func replayLeftClick(like event: NSEvent) {
        backgroundUntil = .now + 1
        leftClick(at: event.locationInWindow)
    }

    /// A real left click on the page at a point in CSS pixels from the viewport's top-left,
    /// so the page sees a trusted click and links keep their target and download behaviour.
    func click(css point: CGPoint) {
        leftClick(at: convert(viewPoint(css: point), to: nil))
    }

    func viewPoint(css point: CGPoint) -> NSPoint {
        let scale = pageZoom * magnification
        return NSPoint(x: point.x * scale, y: isFlipped ? point.y * scale : bounds.height - point.y * scale)
    }

    private func leftClick(at location: NSPoint) {
        for (type, pressure) in [(NSEvent.EventType.leftMouseDown, Float(1)), (.leftMouseUp, 0)] {
            guard let click = NSEvent.mouseEvent(with: type, location: location, modifierFlags: [],
                                                 timestamp: ProcessInfo.processInfo.systemUptime, windowNumber: window?.windowNumber ?? 0,
                                                 context: nil, eventNumber: 0, clickCount: 1, pressure: pressure) else { return }
            type == .leftMouseDown ? mouseDown(with: click) : mouseUp(with: click)
        }
    }
}
