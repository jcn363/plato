import UIKit
import QuartzCore

class PlatoRenderer {
    private weak var view: UIView?
    private var displayLink: CADisplayLink?
    private var pixelBuffer: UnsafeMutablePointer<UInt8>?
    private var bufferSize: Int = 0

    init(view: UIView) {
        self.view = view
        setupDisplayLink()
    }

    deinit {
        displayLink?.invalidate()
        if let buffer = pixelBuffer {
            buffer.deallocate()
        }
    }

    private func setupDisplayLink() {
        displayLink = CADisplayLink(target: self, selector: #selector(renderFrame))
        displayLink?.add(to: .main, forMode: .common)
    }

    @objc private func renderFrame() {
        guard let view = view else { return }

        let screen = UIScreen.main
        let scale = screen.scale
        let width = Int(view.bounds.width * scale)
        let height = Int(view.bounds.height * scale)
        let requiredSize = width * height * 4

        if bufferSize != requiredSize {
            if let buffer = pixelBuffer {
                buffer.deallocate()
            }
            pixelBuffer = UnsafeMutablePointer<UInt8>.allocate(capacity: requiredSize)
            bufferSize = requiredSize
        }

        if let buffer = pixelBuffer {
            if plato_render(buffer, bufferSize) {
                let provider = CGDataProvider(data: Data(bytes: buffer, count: bufferSize) as CFData)
                let cgImage = CGImage(
                    width: width,
                    height: height,
                    bitsPerComponent: 8,
                    bitsPerPixel: 32,
                    bytesPerRow: width * 4,
                    space: CGColorSpaceCreateDeviceRGB(),
                    bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedLast.rawValue),
                    provider: provider!,
                    decode: nil,
                    shouldInterpolate: false,
                    intent: .defaultIntent
                )

                if let image = cgImage {
                    DispatchQueue.main.async {
                        view.layer.contents = image
                    }
                }
            }
        }
    }
}
