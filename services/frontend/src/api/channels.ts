import { reactive } from "vue";
import type { Reactive } from "vue";

import client from "./client";
import { User } from "./users";

export class Channel {
  private static _cache = new Map<number, Channel>();
  public static channels: Reactive<Channel[]> = reactive([]);

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

  public static getChannel(id: number): Channel | null {
    return Channel._cache.get(id) || null;
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
