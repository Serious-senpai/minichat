import client from "./client";
import { User } from "./users";

class FetchMessageQuery {
  public newest: boolean = true;
  public beforeId: number = Number.MAX_SAFE_INTEGER;
  public afterId: number = 0;
  public limit: number = 50;
}

export class Channel {
  private static _cache = new Map<number, Channel>();
  public static channels: Channel[] = [];

  public id: number;
  public name: string;
  public description: string;
  public owner: User;

  public constructor(id: number, name: string, description: string, owner: User) {
    this.id = id;
    this.name = name;
    this.description = description;
    this.owner = owner;
  }

  public async send(content: string): Promise<Message> {
    const response = await client.post<Message>(`/channels/${this.id}/messages`, { content });
    return response.data;
  }

  public async history(config: FetchMessageQuery): Promise<Message[]> {
    const result: Message[] = [];

    let id = config.newest ? config.beforeId : config.afterId;
    while (config.limit > 0) {
      const limit = Math.min(config.limit, 50);
      config.limit -= limit;

      const response = await client.get<Message[]>(
        `/channels/${this.id}/messages`,
        {
          params: {
            newest: config.newest,
            beforeId: config.newest ? id : config.beforeId,
            afterId: config.newest ? config.afterId : id,
            limit,
          },
        },
      );

      const data = response.data;
      if (data.length === 0) {
        break;
      }

      if (config.newest) {
        id = data[data.length - 1].id - 1;
      } else {
        id = data[0].id + 1;
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
    return new Channel(data.id, data.name, data.description, data.owner);
  }

  public static getChannel(id: number): Channel | null {
    return Channel._cache.get(id) || null;
  }

  public static async fetchChannel(id: number): Promise<Channel> {
    const response = await client.get<Channel>(`/channels/${id}`);
    const channel = response.data;
    Channel._cache.set(channel.id, channel);
    return channel;
  }

  public static async query(): Promise<Channel[]> {
    const response = await client.get<Channel[]>("/channels");
    const channels = response.data;
    for (const channel of channels) {
      Channel._cache.set(channel.id, channel);
    }

    channels.sort((a, b) => a.id - b.id);
    Channel.channels = channels;
    return channels;
  }

  public static async create(name: string, description: string): Promise<Channel> {
    const response = await client.post<Channel>(
      "/channels",
      {
        name,
        description,
      },
    );

    return response.data;
  }
}

export class Message {
  public id: number;
  public content: string;
  public author: User;
  public channel: Channel;

  public constructor(id: number, content: string, author: User, channel: Channel) {
    this.id = id;
    this.content = content;
    this.author = author;
    this.channel = channel;
  }
}
