import { basename, extname, parse } from "node:path";
import { Injectable, Logger, type OnModuleInit } from "@nestjs/common";
import { InjectModel } from "@nestjs/mongoose";
import * as imageThumbnail from "image-thumbnail";
import { getIV } from "../utils/encryption";
import md5 from "../utils/md5";
import { Storage, type StorageDocument } from "./schemas/storage.schema";
import LocalStorage from "./storage.local";

const thumbnail_extension_allowlist = [
  "image/png",
  "image/jpeg",
  "image/webp",
  "image/avif",
  "image/tiff",
  "image/gif",
  // 'image/svg+xml', // not supported
];

@Injectable()
export class StorageService implements OnModuleInit {
  private readonly localStorage = new LocalStorage();
  private readonly logger = new Logger(StorageService.name);

  constructor(@InjectModel(Storage.name) private storageModel: any) {}

  onModuleInit() {
    this.logger.log("StorageService dependencies initialized");
  }

  async find(query = {}): Promise<Storage[]> {
    return this.storageModel.find(query).exec();
  }

  async findOne(query = {}): Promise<any> {
    return this.storageModel.findOne(query).lean();
  }

  async deleteOne(id: string, userId: string): Promise<Storage> {
    return this.storageModel.findOneAndRemove({ _id: id, userId });
  }

  async updateOne(
    id: string,
    userId: string,
    updater = {},
    options = {},
  ): Promise<Storage> {
    return this.storageModel.findOneAndUpdate(
      {
        _id: id,
        userId,
      },
      updater,
      options,
    );
  }

  async deleteAll(): Promise<any> {
    return this.storageModel.deleteMany({});
  }

  async createFolder(folder = {}): Promise<StorageDocument> {
    return this.storageModel.createByFolder(folder);
  }

  async getPath(fileId, userId, items = []): Promise<StorageDocument[]> {
    const doc = await this.findOne({
      _id: fileId,
      userId,
      trashed: false,
    });
    const parentId = doc?.parentId;
    const pathItems =
      !parentId || parentId === "root"
        ? doc
          ? [doc]
          : []
        : await this.getPath(parentId, userId, [doc]);

    return items.concat(pathItems);
  }

  async generateThumbnail(
    file,
    docId: string,
    userId: string,
    parentId: "root",
  ): Promise<void> {
    const isInAllowlist = thumbnail_extension_allowlist.includes(
      file?.mimetype?.toLowerCase(),
    );

    if (!docId || !file || !isInAllowlist) {
      return;
    }

    try {
      const thumbnail = await imageThumbnail(file?.buffer);
      const hash = await md5(thumbnail);
      const iv = getIV();
      const { name: fileName } = parse(file.originalname);
      const fileExtname = extname(file.originalname);
      const originalname = `${fileName}_thumbnail${fileExtname}`;
      const thumbnailObj = {
        size: thumbnail.length,
        originalname,
        hash,
        iv,
        type: "thumbnail",
      };
      let doc = await this.findOne({
        MD5Hash: hash,
        userId,
        parentId,
      });
      const fileDoc = await this.storageModel.findByHash(hash);

      if (!doc) {
        doc = await this.storageModel.createByFile({
          ...thumbnailObj,
          userId,
          parentId,
        });
      }

      if (!fileDoc) {
        await this.localStorage.store(doc.MD5Hash, thumbnail, iv);
      } else {
        this.logger.log(
          `Stored thumbnail file ${doc._id} is matched, no new file stored`,
        );
      }

      await this.updateOne(docId, userId, { thumbnail: doc._id });
    } catch (error) {
      this.logger.error(`Generate thumbnail error: ${error}`);

      return error;
    }
  }

  async store(
    files,
    userId: string,
    parentId: "root",
    generateThumbnail = true,
  ): Promise<Record<string, string>> {
    const fields = Object.keys(files || {});

    if (fields.length <= 0) {
      this.logger.error("No file data found from the request");
    }

    const fileIDs = {};

    for (const field of fields) {
      const file = files[field];
      const fileBuffer = file?.buffer;
      const hash = await md5(fileBuffer);
      const iv = getIV();

      file.hash = hash;
      file.iv = iv;

      let doc = await this.findOne({
        MD5Hash: hash,
        userId,
        parentId,
      });
      const fileDoc = await this.storageModel.findByHash(hash);

      if (!doc) {
        doc = await this.storageModel.createByFile({
          ...file,
          userId,
          parentId,
        });

        if (generateThumbnail) {
          await this.generateThumbnail(file, doc._id, userId, parentId);
        }
      } else if (doc.trashed) {
        await this.updateOne(doc._id, userId, {
          name: file.originalname,
          baseName: basename(file.originalname),
          extName: extname(file.originalname),
          trashed: false,
        });
      } else if (doc.name !== file.originalname) {
        await this.updateOne(doc._id, userId, {
          name: file.originalname,
          baseName: basename(file.originalname),
          extName: extname(file.originalname),
        });
      }

      if (!fileDoc) {
        await this.localStorage.store(doc.MD5Hash, fileBuffer, iv);
      } else {
        this.logger.log(
          `Stored file ${doc._id} is matched, no new file stored`,
        );
      }

      fileIDs[field] = doc._id;
    }

    return fileIDs;
  }

  async getFiles(query = {}, pagination = {}): Promise<any> {
    const files = await this.storageModel.getFiles(query, pagination);

    return files;
  }

  async getFile(
    id: string,
    userId: string,
  ): Promise<{
    stream: any;
    doc?: any;
  }> {
    const doc = await this.findOne({
      _id: id,
      userId,
    });

    if (!doc) {
      return {
        stream: "",
      };
    }

    const stream = this.localStorage.fetch(doc.MD5Hash, doc.iv);

    if (!stream) {
      this.logger.error(
        `Found file with id ${doc._id} in database, but no file found on file system`,
      );

      return {
        stream: "",
      };
    }

    return {
      stream,
      doc,
    };
  }

  async removeFileTemporary(id: string, userId: string): Promise<any> {
    const doc = await this.findOne({
      _id: id,
      userId,
    });

    this.logger.log(
      `Storage service: - [remove - temporary] - the file id => ${id}`,
    );
    this.logger.log(
      `Storage service: - [remove - temporary] - the file info => ${JSON.stringify(doc)}`,
    );

    if (!doc) {
      return "";
    }

    await this.updateOne(id, userId, { trashed: true });

    return id;
  }

  async removeFile(id: string, userId: string): Promise<any> {
    const doc = await this.findOne({
      _id: id,
      userId,
    });

    this.logger.log(`Storage service: - [remove] - the file id => ${id}`);
    this.logger.log(
      `Storage service: - [remove] - the file info => ${JSON.stringify(doc)}`,
    );

    if (!doc) {
      return "";
    }

    try {
      const hash = doc.MD5Hash;
      const files = await this.find({ MD5Hash: doc.MD5Hash });

      await this.deleteOne(id, userId);

      // If the file is not used by other users, remove it
      if (files.length <= 1) {
        await this.localStorage.remove(hash);
      }
    } catch (error) {
      this.logger.error(`Remove file error: ${error}`);

      return error;
    }

    return id;
  }
}
