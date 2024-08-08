export abstract class ManagerModule {
  abstract run(): Promise<void>;

  protected _lastRun: number = 0;
  get lastRun(): number {
    return this._lastRun;
  }
}
