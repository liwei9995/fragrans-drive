import * as fs from "node:fs";
import { dirname, join } from "node:path";
import { Readable } from "node:stream";
import { Logger } from "@nestjs/common";
import { sync } from "mkdirp";
import { decryptStream, encryptBuffer } from "../utils/encryption";

const SUB_DIRECTORY_DEPTH = 3;
const SUB_DIRECTORY_LENGTH = 2;

const clearDir = (p, root) =>
  new Promise((resolve, reject) => {
    fs.stat(p, (err, stat) => {
      if (err) {
        return reject(err);
      }

      if (!stat.isDirectory()) {
        return resolve(false);
      }

      fs.readdir(p, (err, files) => {
        if (err) {
          return reject(err);
        }

        if (!files || files.length <= 0) {
          fs.rmdir(p, (err) => {
            if (err) {
              return reject(err);
            }

            const parent = dirname(p);

            if (parent === root) {
              return resolve(true);
            }

            clearDir(parent, root)
              .then((success) => resolve(success))
              .catch((err) => reject(err));
          });
        }
      });
    });
  });

const pipeStream = (readable, writable) =>
  new Promise((resolve, reject) => {
    writable.on("error", (err) => reject(err));
    readable.on("error", (err) => reject(err));
    readable.on("end", () => resolve(true));
    readable.pipe(writable);
  });

const getReadableStream = (fileStream): Readable => {
  return new Readable({
    read() {
      this.push(fileStream);
      this.push(null);
    },
  });
};

class LocalStorage {
  options: {
    path: string;
    flatten: boolean;
    subDirectoryDepth: number;
    subDirectoryLength: number;
  };
  pathRegex: RegExp;
  private readonly logger = new Logger(LocalStorage.name);

  constructor(opts: { path?: string } = {}) {
    const defaultPath =
      process.env.STORAGE_DESTINATION ||
      join(__dirname, "../../bucket/storage");
    this.options = Object.assign({}, opts, {
      path: opts.path ?? defaultPath,
      flatten: false,
      subDirectoryDepth: SUB_DIRECTORY_DEPTH,
      subDirectoryLength: SUB_DIRECTORY_LENGTH,
    });

    let r = "";

    for (let i = 0; i < this.options.subDirectoryDepth; i++) {
      r += `(.{${this.options.subDirectoryLength}})`;
    }
    this.pathRegex = new RegExp(r);

    this.init();
  }

  async init() {
    const { flatten, path: storePath, subDirectoryDepth } = this.options;
    const configPath = join(storePath, "./.config.json");
    let ok = fs.existsSync(storePath);

    if (!ok) {
      sync(storePath);
    }
    ok = fs.existsSync(configPath);

    let config = null;

    if (!ok) {
      config = {
        depth: subDirectoryDepth,
        flatten,
      };
      fs.writeFileSync(configPath, JSON.stringify(config));
    } else {
      config = await import(configPath);
    }

    if (config.flatten !== flatten || config.depth !== subDirectoryDepth) {
      throw new Error(
        `Configuration of target file store at "${storePath}" is not matched, please use another one`,
      );
    }

    this.logger.log(`Local-store: ready at ${storePath}`);
  }

  getPath(id) {
    id = id.toString ? id.toString() : id;
    const arr = this.pathRegex.exec(id);

    arr.shift();

    return join(this.options.path, arr.join("/"), id);
  }

  exists(id) {
    const p = this.getPath(id);

    return fs.existsSync(p);
  }

  fetch(id, iv?: string) {
    const p = this.getPath(id);
    const exists = this.exists(id);

    return exists
      ? iv
        ? decryptStream(id, iv, fs.createReadStream(p))
        : fs.createReadStream(p)
      : null;
  }

  async store(id: string, buffer, iv?: string) {
    const ok = this.exists(id);

    if (ok) {
      return false;
    }

    const p = this.getPath(id);
    const dir = dirname(p);

    if (!fs.existsSync(dir)) {
      sync(dir);
    }

    const ws = fs.createWriteStream(p);

    await pipeStream(
      getReadableStream(iv ? encryptBuffer(id, iv, buffer) : buffer),
      ws,
    );
    this.logger.log(`Local-store: ${id} stored at ${p}`);

    return true;
  }

  async remove(id) {
    const p = this.getPath(id);

    this.logger.log(`Local-store: the path of file need to remove => ${p}`);

    try {
      fs.unlinkSync(p);
    } catch (err) {
      this.logger.log(`Local-store: remove file err => ${err}`);
      if (err.code !== "ENOENT") {
        throw err;
      }
      this.logger.warn(
        `Local-store: intend to remove file at path ${p}, but it does not exist.`,
      );
    }

    try {
      clearDir(dirname(p), this.options.path);
    } catch (err) {
      this.logger.log(`Local-store: clear dir error => ${err}`);
    }

    this.logger.log(`Local-store: the file(${id}) removed from ${p}`);
  }
}

export default LocalStorage;
