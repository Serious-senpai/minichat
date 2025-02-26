import { snowflakeTime } from "./utils";

export default class Snowflake {
  public readonly id: bigint;

  public constructor(id: bigint) {
    this.id = id;
  }

  public get createdAt(): Date {
    return snowflakeTime(this.id);
  }

  public static ensure(data: Snowflake): Snowflake {
    return new Snowflake(BigInt(data.id));
  }
}
