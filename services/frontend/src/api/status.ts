export class Status {
  public success: boolean;
  public message: string;

  public constructor(success: boolean, message: string) {
    this.success = success;
    this.message = message;
  }
}
