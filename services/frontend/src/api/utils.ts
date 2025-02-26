export class Settings {
  public readonly epoch: Date;

  public constructor(epoch: Date) {
    this.epoch = epoch;
  }
}

let settings: Settings | null = null;

export async function importSetup(): Promise<void> {
  try {
    const response = await fetch("/setup.json");
    if (response.status === 200) {
      const data = await response.json();
      settings = new Settings(new Date(data.epoch));
    } else {
      await importSetup();
    }
  } catch {
    await importSetup();
  }
}

export function snowflakeTime(id: bigint): Date {
  if (settings === null) return new Date(0);
  return new Date(settings.epoch.getTime() + Number(id >> 16n));
}
