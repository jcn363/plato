import UIKit

class PlatoViewController: UIViewController {
    private var renderer: PlatoRenderer?

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .white

        let screen = UIScreen.main
        let scale = screen.scale
        let width = UInt32(screen.bounds.width * scale)
        let height = UInt32(screen.bounds.height * scale)

        if plato_init(width, height) {
            renderer = PlatoRenderer(view: view)
        }
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        for touch in touches {
            let location = touch.location(in: view)
            let scale = UIScreen.main.scale
            let x = Int32(location.x * scale)
            let y = Int32(location.y * scale)
            let id = Int32(bitPattern: touch.hashValue)

            plato_touch_down(id, x, y)
        }
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        for touch in touches {
            let location = touch.location(in: view)
            let scale = UIScreen.main.scale
            let x = Int32(location.x * scale)
            let y = Int32(location.y * scale)
            let id = Int32(bitPattern: touch.hashValue)

            plato_touch_move(id, x, y)
        }
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        for touch in touches {
            let location = touch.location(in: view)
            let scale = UIScreen.main.scale
            let x = Int32(location.x * scale)
            let y = Int32(location.y * scale)
            let id = Int32(bitPattern: touch.hashValue)

            plato_touch_up(id, x, y)
        }
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        for touch in touches {
            let location = touch.location(in: view)
            let scale = UIScreen.main.scale
            let x = Int32(location.x * scale)
            let y = Int32(location.y * scale)
            let id = Int32(bitPattern: touch.hashValue)

            plato_touch_up(id, x, y)
        }
    }

    deinit {
        plato_deinit()
    }
}
