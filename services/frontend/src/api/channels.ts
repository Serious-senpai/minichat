import JSONBigInt from "json-bigint";

import client from "./client";
import User from "./users";
import Snowflake from "./snowflake";

class FetchMessageQuery {
  public newest: boolean = true;
  public beforeId: bigint = 1n << 63n;
  public afterId: bigint = 0n;
  public limit: number = 50;
}

export class Channel extends Snowflake {
  private static readonly _cache = new Map<bigint, Channel>();
  public static channels: Channel[] = [];

  public readonly name: string;
  public readonly description: string;
  public readonly owner: User;

  public constructor(id: bigint, name: string, description: string, owner: User) {
    super(id);
    this.name = name;
    this.description = description;
    this.owner = owner;
  }

  public async send(content: string): Promise<Message> {
    const response = await client.post<Message>(
      `/channels/${this.id}/messages`,
      { content },
      { transformResponse: [data => data] },
    );
    return response.data;
  }

  public async history(config: FetchMessageQuery): Promise<Message[]> {
    const result: Message[] = [];

    let id = config.newest ? config.beforeId : config.afterId;
    while (config.limit > 0) {
      const limit = Math.min(config.limit, 50);
      config.limit -= limit;

      const response = await client.get<string>(
        `/channels/${this.id}/messages`,
        {
          params: {
            newest: config.newest,
            before_id: config.newest ? id : config.beforeId,
            after_id: config.newest ? config.afterId : id,
            limit,
          },
          transformResponse: [data => data],
        },
      );

      const data = JSONBigInt.parse(response.data) as Message[];
      if (data.length === 0) break;

      for (let i = 0; i < data.length; i++) {
        data[i] = Message.ensure(data[i]);
      }

      if (config.newest) {
        id = data[data.length - 1].id - 1n;
      } else {
        id = data[0].id + 1n;
      }

      result.push(...data);
    }

    return result;
  }

  public poll(handler: (m: Message) => void): WebSocket {
    const ws = client.websocket(`/channels/${this.id}/ws`);
    ws.onmessage = (event) => {
      const message = JSON.parse(event.data) as Message;
      handler(message);
    };
    return ws;
  }

  public static ensure(data: Channel): Channel {
    return new Channel(BigInt(data.id), data.name, data.description, User.ensure(data.owner));
  }

  public static getChannel(id: bigint): Channel | null {
    return Channel._cache.get(id) || null;
  }

  public static async fetchChannel(id: bigint): Promise<Channel> {
    const response = await client.get<string>(
      `/channels/${id}`,
      { transformResponse: [data => data] },
    );
    const channel = JSONBigInt.parse(response.data) as Channel;
    Channel._cache.set(channel.id, channel);
    return channel;
  }

  public static async query(): Promise<Channel[]> {
    const response = await client.get<string>(
      "/channels",
      { transformResponse: [data => data] },
    );
    const channels = JSONBigInt.parse(response.data) as Channel[];
    for (const channel of channels) {
      Channel._cache.set(channel.id, channel);
    }

    channels.sort((a, b) => Number(a.id - b.id));
    Channel.channels = channels;
    return channels;
  }

  public static async create(name: string, description: string): Promise<Channel> {
    const response = await client.post<string>(
      "/channels",
      {
        name,
        description,
      },
      { transformResponse: [data => data] },
    );

    return JSONBigInt.parse(response.data) as Channel;
  }
}

export class Message extends Snowflake {
  public readonly content: string;
  public readonly author: User;
  public readonly channel: Channel;

  public constructor(id: bigint, content: string, author: User, channel: Channel) {
    super(id);
    this.content = content;
    this.author = author;
    this.channel = channel;
  }

  public static ensure(data: Message): Message {
    return new Message(BigInt(data.id), data.content, User.ensure(data.author), Channel.ensure(data.channel));
  }
}
