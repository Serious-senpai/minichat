import axios from "axios";
import type { AxiosRequestConfig, AxiosResponse } from "axios";

import { User } from "./users";

class AccountToken {
  public access_token: string;
  public token_type: string;

  public constructor(access_token: string, token_type: string) {
    this.access_token = access_token;
    this.token_type = token_type;
  }
}

class Client {
  private static readonly _HTTP_URL = "http://" + import.meta.env.VITE_APP_BASE_API_URL;
  private static readonly _WS_URL = "ws://" + import.meta.env.VITE_APP_BASE_API_URL;
  private static readonly _http = axios.create(
    {
      baseURL: Client._HTTP_URL,
    }
  );

  private _user: User | null = null;
  public get user(): User | null {
    return this._user;
  }

  public async login(): Promise<boolean> {
    const username = localStorage.getItem("username");
    const password = localStorage.getItem("password");

    try {
      if (username && password) {
        const form = new FormData();
        form.set("grant_type", "password");
        form.set("username", username);
        form.set("password", password);

        const response = await this.post<AccountToken>(
          "/auth/token",
          form,
          {
            headers: {
              "Content-Type": "application/x-www-form-urlencoded",
            }
          },
        );

        if (response.status === 200) {
          localStorage.setItem("access_token", response.data.access_token);
          localStorage.setItem("username", username);
          localStorage.setItem("password", password);

          const me = await this.get<User>("/auth/@me");
          this._user = me.data;

          return true;
        }
      };
    } catch {
      // pass
    }

    this.logout();
    return false;
  }

  public logout(): void {
    localStorage.removeItem("access_token");
    localStorage.removeItem("username");
    localStorage.removeItem("password");
    this._user = null;
  }

  private _addAuthorizationHeader<D>(config?: AxiosRequestConfig<D>): AxiosRequestConfig<D> | undefined {
    const accessToken = localStorage.getItem("access_token");
    if (accessToken) {
      if (!config) {
        config = {};
      }
      if (!config.headers) {
        config.headers = {};
      }

      config.headers.Authorization = `Bearer ${accessToken}`;
    }

    return config;
  }

  public get<T, R = AxiosResponse<T>, D = unknown>(url: string, config?: AxiosRequestConfig<D>): Promise<R> {
    config = this._addAuthorizationHeader(config);
    return Client._http.get<T, R, D>(url, config);
  }

  public post<T, R = AxiosResponse<T>, D = unknown>(url: string, data?: D, config?: AxiosRequestConfig<D>): Promise<R> {
    config = this._addAuthorizationHeader(config);
    return Client._http.post<T, R, D>(url, data, config);
  }
}

const client = new Client();

export default client;
