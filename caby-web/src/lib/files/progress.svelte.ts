const WEIGHT_FACTOR = 5;

export class Progress {
	public progress: number = $state(0);
	public total: number = $state(0);

	// values to calculate moving average rate per second
	public rate: number = $state(0); // bytes per second
	private last_progress_seconds: number = 0;
	private progress_ct = 0;

	// public eta: number = $derived(0);

	constructor(total?: number) {
		this.total = total || 0;
	}

	// average = average + (value - average) / min(counter, FACTOR)
	public setProgress(progress: number) {
		const now_seconds = performance.now() / 1000;
		if (this.progress_ct > 1) {
			const rate = (progress - this.progress) / (now_seconds - this.last_progress_seconds);
			this.rate = Math.floor(
				this.rate + (rate - this.rate) / Math.min(this.progress_ct, WEIGHT_FACTOR)
			);
		}

		this.progress_ct++;
		this.progress = progress;
		this.last_progress_seconds = now_seconds;
	}

	public addProgress(progress: number) {
		this.setProgress(this.progress + progress);
	}

	public addTotal(total: number) {
		this.total += total;
	}

	public reset() {
		this.progress = 0;
		this.total = 0;
	}
}

// export class CumulativeProgress {
// 	public progress: number = $state(0);
// 	public total: number = $state(0);

// 	// values to calculate moving average rate per second
// 	public average_rate: number = $state(0); // bytes per second
// 	private progress_ct = 0;

// 	constructor(total?: number) {
// 		this.total = total || 0;
// 	}

// 	// average = average + (value - average) / min(counter, FACTOR)
// 	public setProgress(progress: number) {
// 		const now_seconds = performance.now() / 1000;
// 		if (this.last_progress_seconds > 0) {
// 			const rate = (progress - this.progress) / (now_seconds - this.last_progress_seconds);
// 			this.rate = Math.floor(
// 				this.rate + (rate - this.rate) / Math.min(this.progress_ct, WEIGHT_FACTOR)
// 			);
// 		}

// 		this.progress_ct++;
// 		this.progress = progress;
// 		this.last_progress_seconds = now_seconds;
// 	}

// 	public addProgress(progress: number) {
// 		this.setProgress(this.progress + progress);
// 	}

// 	public addTotal(total: number) {
// 		this.total += total;
// 	}
// }
