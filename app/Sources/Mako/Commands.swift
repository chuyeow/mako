import AppKit
import MakoCore

/// A main-menu item the command palette can run. The menu bar is the list of everything
/// Mako can do, so a feature with a menu item is in the palette without more work.
@MainActor
struct MenuCommand {
    let item: NSMenuItem
    /// The menus above it, outermost first, e.g. ["View"].
    let path: [String]

    var name: String { item.title }

    /// The item's shortcut as the menu bar draws it, e.g. ⌥⌘T; empty without one, or for a
    /// key with no printable name (AppKit's own items use function-key characters).
    var keys: String {
        let key = item.keyEquivalent
        let name = switch key {
        case " ": "Space"
        case "\r": "↩"
        case "\t": "⇥"
        case "\u{1b}": "⎋"
        case "\u{8}", "\u{7f}": "⌫"
        default: key.unicodeScalars.allSatisfy { $0.isASCII && $0.properties.isGraphemeBase && !$0.properties.isWhitespace } ? key.uppercased() : ""
        }
        guard !name.isEmpty else { return "" }
        let mods = item.keyEquivalentModifierMask
        var text = mods.contains(.function) ? "fn " : ""
        if mods.contains(.control) { text += "⌃" }
        if mods.contains(.option) { text += "⌥" }
        if mods.contains(.shift) || key != key.lowercased() { text += "⇧" }
        if mods.contains(.command) { text += "⌘" }
        return text + name
    }

    /// Every item `menu` and its submenus enable right now, in menu order, except the
    /// palette's own. Enabling is checked against the current first responder, so call this
    /// before the omnibox takes focus.
    static func all(in menu: NSMenu, path: [String] = []) -> [MenuCommand] {
        menu.update()
        return menu.items.flatMap { item -> [MenuCommand] in
            if item.isSeparatorItem || item.isHidden { return [] }
            if let submenu = item.submenu { return all(in: submenu, path: path + [item.title]) }
            guard let action = item.action, action != #selector(Browser.showCommands), item.isEnabled else { return [] }
            return [MenuCommand(item: item, path: path)]
        }
    }

    /// `commands` that match `query`, best first; the core ranks them.
    static func rank(_ query: String, _ commands: [MenuCommand]) -> [MenuCommand] {
        let list = commands.map { ["name": $0.name, "path": $0.path.joined(separator: " ")] }
        guard let data = try? JSONSerialization.data(withJSONObject: list),
              let reply = Core.take(mako_rank_commands(query, String(decoding: data, as: UTF8.self))),
              let order = try? JSONSerialization.jsonObject(with: Data(reply.utf8)) as? [Int] else { return [] }
        return order.filter(commands.indices.contains).map { commands[$0] }
    }

    /// Sends its action as its menu would. Not through the menu: the Trails menu is rebuilt
    /// whenever a page loads or retitles, which takes its old items out of it.
    func perform() {
        guard let action = item.action else { return }
        NSApp.sendAction(action, to: item.target, from: item)
    }
}
