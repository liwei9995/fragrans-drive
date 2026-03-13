import { Injectable } from "@nestjs/common";
import * as config from "config";

@Injectable()
export class ConfigService {
  get(key: string): any {
    return config.get(key);
  }
}
