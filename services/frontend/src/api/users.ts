export class User {
  public id: number;
  public username: string;
  public permissions: number;

  public constructor(id: number, username: string, permissions: number) {
    this.id = id;
    this.username = username;
    this.permissions = permissions;
  }
}
