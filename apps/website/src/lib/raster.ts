// Rasterise raw RGBA8 pixels to a PNG object URL for an <img>. The caller owns
// the URL and must revoke it.
export function rgbaToUrl(rgba: Uint8Array, width: number, height: number): Promise<string> {
	const canvas = document.createElement('canvas');
	canvas.width = width;
	canvas.height = height;
	const ctx = canvas.getContext('2d');
	if (!ctx) return Promise.reject(new Error('2D canvas unavailable'));

	const image = new ImageData(new Uint8ClampedArray(rgba), width, height);
	ctx.putImageData(image, 0, 0);

	return new Promise((resolve, reject) => {
		canvas.toBlob((blob) => {
			if (blob) resolve(URL.createObjectURL(blob));
			else reject(new Error('Frame rasterisation failed'));
		}, 'image/png');
	});
}
