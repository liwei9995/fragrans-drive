import type { ReadStream } from "node:fs";
import { basename, extname } from "node:path";
import {
  BadRequestException,
  Body,
  Controller,
  Delete,
  Get,
  Logger,
  NotFoundException,
  Param,
  Post,
  Put,
  Query,
  Request,
  Response,
  StreamableFile,
  UnauthorizedException,
  UploadedFiles,
  UseInterceptors,
} from "@nestjs/common";
import type { JwtService } from "@nestjs/jwt";
import { AnyFilesInterceptor } from "@nestjs/platform-express";
import * as contentDisposition from "content-disposition";
import type { Express } from "express";
import { Types } from "mongoose";
import { Public } from "../common/decorator/auth.decorator";
import { Roles } from "../common/decorator/roles.decorator";
import { Role } from "../common/enums/role.enum";
import type { ConfigService } from "../config/config.service";
import type { StorageService } from "./storage.service";

const desensitize = (file) => ({
  id: file?._id,
  name: file?.name,
  baseName: file?.baseName,
  extName: file?.extName,
  mimeType: file?.mimeType,
  encoding: file?.encoding,
  size: file?.size,
  parentId: file?.parentId,
  type: file?.type,
  thumbnail: file?.thumbnail,
  url: file?.url,
  createdAt: file?.createdAt,
  updatedAt: file?.updatedAt,
});

@Controller({
  path: "storage",
  version: "1",
})
export class StorageController {
  constructor(
    private configService: ConfigService,
    private jwtService: JwtService,
    private storageService: StorageService,
  ) {}
  private readonly logger = new Logger(StorageController.name);

  getDomain() {
    const host = this.configService.get("http.host");
    const port = this.configService.get("http.port");
    const domain =
      this.configService.get("drive.domain") || `http://${host}:${port}`;

    return domain;
  }

  @Post("upload")
  @Roles(Role.User)
  @UseInterceptors(
    AnyFilesInterceptor({
      fileFilter(_req, file, callback) {
        file.originalname = Buffer.from(file.originalname, "latin1").toString(
          "utf8",
        );
        callback(null, true);
      },
    }),
  )
  async uploadFile(
    @Request() req,
    @UploadedFiles() files: Array<Express.Multer.File>,
  ) {
    const userId = req.user?.userId;
    const parentId = req?.body?.parentId || "root";

    if (!Types.ObjectId.isValid(userId) || files.length <= 0) {
      throw new BadRequestException();
    }

    const ids = await this.storageService.store(files, userId, parentId);

    return ids;
  }

  @Post("folder")
  @Roles(Role.User)
  async createFolder(@Request() req, @Body() body): Promise<any> {
    const userId = req.user?.userId;
    const { name, parentId, type } = body;

    if (!Types.ObjectId.isValid(userId)) {
      throw new BadRequestException();
    }

    const doc = await this.storageService.findOne({
      name,
      parentId,
      userId,
      type,
      trashed: false,
    });

    if (doc?.name) {
      return {
        _id: doc._id,
        name: doc.name,
        parentId: doc.parentId,
        type: doc.type,
        createdAt: doc.createdAt,
        updatedAt: doc.updatedAt,
        exist: true,
      };
    }

    const folder = await this.storageService.createFolder({
      originalname: name,
      parentId,
      type,
      userId,
    });

    return {
      _id: folder._id,
      name: folder.name,
      parentId: folder.parentId,
      type: folder.type,
      createdAt: folder.createdAt,
      updatedAt: folder.updatedAt,
    };
  }

  @Post("list")
  @Roles(Role.User)
  async getFiles(@Request() req, @Body() body): Promise<Storage[]> {
    const userId = req.user?.userId;

    if (!Types.ObjectId.isValid(userId)) {
      throw new BadRequestException();
    }

    const { query = {}, pagination } = body;

    const files = await this.storageService.getFiles(
      {
        ...query,
        userId,
        trashed: false,
        type: {
          $in: ["file", "folder"],
        },
      },
      pagination,
    );

    const domain = this.getDomain();
    const token = this.jwtService.sign({ userId });
    const docs = files?.docs?.map((file) => {
      const doc = desensitize(file);

      if (doc.thumbnail) {
        doc.thumbnail = `${domain}/v1/storage/${doc.thumbnail}?token=${token}`;
        doc.url = `${domain}/v1/storage/${doc.id}?token=${token}`;
      } else if (doc.mimeType?.startsWith("video/")) {
        doc.url = `${domain}/v1/storage/${doc.id}?token=${token}`;
      }

      return doc;
    });

    return {
      ...files,
      docs,
    };
  }

  @Post("path")
  @Roles(Role.User)
  async getPath(@Request() req, @Body() body): Promise<any[]> {
    const userId = req.user?.userId;
    const { fileId } = body;

    if (!Types.ObjectId.isValid(userId) || !Types.ObjectId.isValid(fileId)) {
      throw new BadRequestException();
    }

    const files = await this.storageService.getPath(fileId, userId);

    return files.reverse().map((file) => desensitize(file));
  }

  @Post("move")
  @Roles(Role.User)
  async moveFile(@Request() req, @Body() body): Promise<any> {
    const userId = req.user?.userId;
    const { fileId, parentId } = body;

    if (
      !Types.ObjectId.isValid(userId) ||
      !Types.ObjectId.isValid(fileId) ||
      !parentId
    ) {
      throw new BadRequestException();
    }

    const result = await this.storageService.updateOne(
      fileId,
      userId,
      { parentId },
      { new: true },
    );

    return result;
  }

  @Post("download/url")
  @Roles(Role.User)
  async getDownloadUrl(
    @Request() req,
    @Body() body,
  ): Promise<BadRequestException | any> {
    const userId = req.user?.userId;
    const { fileId } = body;

    if (!Types.ObjectId.isValid(userId) || !Types.ObjectId.isValid(fileId)) {
      throw new BadRequestException();
    }

    const doc = await this.storageService.findOne({
      _id: fileId,
      trashed: false,
    });

    if (!doc) {
      throw new NotFoundException();
    }

    const token = this.jwtService.sign({ userId });
    const domain = this.getDomain();
    const url = `${domain}/v1/storage/${doc._id}?token=${token}`;

    return {
      ...desensitize(doc),
      url,
    };
  }

  @Public()
  @Get(":id")
  async getFile(
    @Param("id") id: string,
    @Query() query,
    @Response({ passthrough: true }) res,
  ): Promise<BadRequestException | StreamableFile | string> {
    const token = query?.token;
    const payload = this.jwtService.decode(token) as {
      userId?: string;
      exp: number;
    };
    const userId = payload?.userId;
    const exp = payload?.exp;
    const timestamp = parseInt((Date.now() / 1000).toString(), 10);
    const isExpired = exp - timestamp < 0;

    if (isExpired) {
      throw new UnauthorizedException();
    }

    if (!Types.ObjectId.isValid(id) || !Types.ObjectId.isValid(userId)) {
      throw new BadRequestException();
    }

    const { doc, stream } = await this.storageService.getFile(id, userId);
    const filename = doc?.name
      ? doc?.name
      : doc?.extName
        ? doc?._id + doc?.extName
        : null;

    if (filename) {
      this.logger.log(`download file with filename: ${filename}`);
      res.set({
        "Content-Disposition": contentDisposition(filename),
      });
    }

    if (doc) {
      res.set({
        "Content-Type": doc?.mimeType,
        ETag: doc?.MD5Hash,
      });
    }

    const file = stream ? new StreamableFile(stream as ReadStream) : "";

    return file;
  }

  @Put(":id")
  @Roles(Role.User)
  async updateFile(
    @Param("id") id: string,
    @Request() req,
    @Body() body,
  ): Promise<any> {
    const userId = req.user?.userId;

    if (!Types.ObjectId.isValid(id) || !Types.ObjectId.isValid(userId)) {
      throw new BadRequestException();
    }

    const doc = await this.storageService.findOne({
      userId,
      name: body.name,
      parentId: body.parentId || "root",
      type: body.type,
      trashed: false,
    });

    if (doc?.name) {
      return {
        _id: doc._id,
        name: doc.name,
        parentId: doc.parentId,
        type: doc.type,
        createdAt: doc.createdAt,
        updatedAt: doc.updatedAt,
        exist: true,
      };
    }

    const baseName = basename(body.name);
    const extName = extname(body.name);
    const result = await this.storageService.updateOne(
      id,
      userId,
      {
        name: body.name,
        baseName,
        extName,
      },
      { new: true },
    );

    return result;
  }

  @Delete(":id")
  @Roles(Role.User)
  async removeFile(
    @Param("id") id: string,
    @Request() req,
  ): Promise<BadRequestException | string> {
    const userId = req.user?.userId;

    if (!Types.ObjectId.isValid(id) || !Types.ObjectId.isValid(userId)) {
      throw new BadRequestException();
    }

    const result = await this.storageService.removeFileTemporary(id, userId);

    return result;
  }
}
