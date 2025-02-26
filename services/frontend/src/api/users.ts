import Snowflake from "./snowflake";

export default class User extends Snowflake {
  public readonly username: string;
  public readonly permissions: number;

  public constructor(id: bigint, username: string, permissions: number) {
    super(id);
    this.username = username;
    this.permissions = permissions;
  }

  public static ensure(data: User): User {
    return new User(data.id, data.username, data.permissions);
  }
}
