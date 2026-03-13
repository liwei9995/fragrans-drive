import { Module } from "@nestjs/common";
import { JwtModule } from "@nestjs/jwt";
import { MongooseModule } from "@nestjs/mongoose";
import { MulterModule } from "@nestjs/platform-express";
import { ConfigModule } from "../config/config.module";
import { jwtConstants } from "./constants";
import StorageClass from "./models/storage.model";
import { Storage, StorageSchema } from "./schemas/storage.schema";
import { StorageController } from "./storage.controller";
import { StorageService } from "./storage.service";

@Module({
  imports: [
    MulterModule,
    MongooseModule.forFeatureAsync([
      {
        name: Storage.name,
        useFactory: () => {
          const schema = StorageSchema;

          schema.index({
            createdAt: -1,
          });
          // eslint-disable-next-line @typescript-eslint/no-var-requires
          schema.plugin(require("mongoose-paginate-v2"));
          schema.loadClass(StorageClass);

          return schema;
        },
      },
    ]),
    ConfigModule.register(),
    JwtModule.register({
      secret: jwtConstants.secret,
      signOptions: { expiresIn: jwtConstants.expiresIn },
    }),
  ],
  controllers: [StorageController],
  providers: [StorageService],
  exports: [StorageService],
})
export class StorageModule {}
