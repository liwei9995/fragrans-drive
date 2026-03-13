import { basename, extname } from "node:path";

export type File = {
  size?: number;
  originalname?: string;
  mimetype?: string;
  type?: string;
  encoding?: string;
  hash?: string;
  iv?: string;
  parentId?: string;
  userId: string;
  thumbnail?: string;
};

export default class StorageClass {
  static findOne: any;
  static paginate: any;
  static create: any;

  static async createByFile(file = {} as File) {
    const obj = {
      size: file.size,
      name: file.originalname,
      extName: extname(file.originalname),
      baseName: basename(file.originalname),
      mimeType: file.mimetype,
      encoding: file.encoding,
      MD5Hash: file.hash,
      iv: file.iv,
      parentId: file.parentId,
      userId: file.userId,
      type: file.type || "file",
      thumbnail: file.thumbnail,
    };

    return StorageClass.create(obj);
  }

  static async createByFolder(folder = {} as File) {
    const obj = {
      name: folder.originalname,
      parentId: folder.parentId,
      userId: folder.userId,
      type: folder.type || "folder",
    };

    return StorageClass.create(obj);
  }

  static async findByHash(hash) {
    return StorageClass.findOne({
      MD5Hash: hash,
    });
  }

  static async getFiles(query = {}, pagination = {}) {
    const files = await StorageClass.paginate(query, pagination);

    return files;
  }
}
