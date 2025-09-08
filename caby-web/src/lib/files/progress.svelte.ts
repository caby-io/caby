import { MAX_UPLOAD_THREADS } from "./upload_manager.svelte";

const WEIGHT_FACTOR = 5;

export class Progress {
	public progress: number = $state(0);
	public total: number = $state(0);

	// values to calculate moving average rate per second
	public rate: number = $state(0); // bytes per second
	private last_progress_seconds: number = 0;
	private progress_ct = 0;

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

export class CombinedProgress {
	public progress: number = $state(0);
	public total: number = $state(0);

	public file_rate: Map<number, number> = new Map();
	public total_rate: number = $state(0); // bytes per second

	// average = average + (value - average) / min(counter, FACTOR)
	public setProgress(progress: number) {
		this.progress = progress;
	}

	public addProgress(progress: number) {
		this.setProgress(this.progress + progress);
	}

	public registerUpload(): number {
		for (let i = 0; i <= MAX_UPLOAD_THREADS; i++) {
			if (this.file_rate.get(i)) {
				continue
			}
			this.file_rate.set(i, 0);
			return i
		}
		throw ("could not register upload");
	}

	public unregisterUpload(id: number) {
		this.file_rate.delete(id);
	}

	public setRate(id: number, rate: number) {
		this.file_rate.set(id, rate);
		let rate_sum = 0;
		this.file_rate.forEach((p) => {
			rate_sum += p
		});

		this.total_rate = Math.floor(this.total_rate + (rate_sum - this.total_rate) / (WEIGHT_FACTOR))
	}

	public addTotal(total: number) {
		this.total += total;
	}

	public reset() {
		this.progress = 0;
		this.total = 0;
		this.file_rate = new Map();
		this.total_rate = 0;
	}
}
