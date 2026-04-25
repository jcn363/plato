import UIKit

class PlatoInput: UIResponder {
    private weak var view: UIView?

    init(view: UIView) {
        self.view = view
        super.init()
    }

    // Helper function to convert touch to stable Int32 identifier
    private func touchId(_ touch: UITouch) -> Int32 {
        // Use ObjectIdentifier for stable touch identity, convert to Int32
        let objectId = ObjectIdentifier(touch)
        return Int32(truncatingIfNeeded: objectId.hashValue)
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let view = view else { return }

        for touch in touches {
            let location = touch.location(in: view)
            let scale = UIScreen.main.scale
            let x = Int32(location.x * scale)
            let y = Int32(location.y * scale)
            let id = touchId(touch)

            plato_touch_down(id, x, y)
        }
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let view = view else { return }

        for touch in touches {
            let location = touch.location(in: view)
            let scale = UIScreen.main.scale
            let x = Int32(location.x * scale)
            let y = Int32(location.y * scale)
            let id = touchId(touch)

            plato_touch_move(id, x, y)
        }
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let view = view else { return }

        for touch in touches {
            let location = touch.location(in: view)
            let scale = UIScreen.main.scale
            let x = Int32(location.x * scale)
            let y = Int32(location.y * scale)
            let id = touchId(touch)

            plato_touch_up(id, x, y)
        }
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        guard let view = view else { return }

        for touch in touches {
            let location = touch.location(in: view)
            let scale = UIScreen.main.scale
            let x = Int32(location.x * scale)
            let y = Int32(location.y * scale)
            let id = touchId(touch)

            plato_touch_up(id, x, y)
        }
    }
}
