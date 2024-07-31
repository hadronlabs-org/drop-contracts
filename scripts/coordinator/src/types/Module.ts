export interface ManagerModule {
  run(): Promise<void>;
  get lastRun(): number;
}
