import { Module } from "@nestjs/common";
import { APP_GUARD, APP_INTERCEPTOR } from "@nestjs/core";
import { MongooseModule } from "@nestjs/mongoose";
import { AppController } from "./app.controller";
import { AppService } from "./app.service";
import { AuthModule } from "./auth/auth.module";
import { JwtAuthGuard } from "./auth/jwt-auth.guard";
import { RolesGuard } from "./common/guard/roles.guard";
import { LoggingInterceptor } from "./common/interceptor/logging.interceptor";
import { ConfigModule } from "./config/config.module";
import { ConfigService } from "./config/config.service";
import { StorageModule } from "./storage/storage.module";
import { UsersModule } from "./users/users.module";

@Module({
  imports: [
    ConfigModule.register(),
    MongooseModule.forRootAsync({
      imports: [ConfigModule.register()],
      useFactory: async (configService: ConfigService) => {
        const envUri = process.env.MONGO_URI;
        if (envUri) {
          return { uri: envUri };
        }
        const dbConfig = configService.get("db.mongo") as {
          username: string;
          password: string;
          database: string;
          url: string;
          port: number;
        };
        const username = dbConfig?.username;
        const password = dbConfig?.password;
        const url = dbConfig?.url;
        const port = dbConfig?.port;
        const database = dbConfig?.database;

        return {
          uri: `mongodb://${username}:${password}@${url}:${port}/${database}?authSource=admin`,
        };
      },
      inject: [ConfigService],
    }),
    AuthModule,
    UsersModule,
    StorageModule,
  ],
  controllers: [AppController],
  providers: [
    AppService,
    {
      provide: APP_INTERCEPTOR,
      useClass: LoggingInterceptor,
    },
    {
      provide: APP_GUARD,
      useClass: JwtAuthGuard,
    },
    {
      provide: APP_GUARD,
      useClass: RolesGuard,
    },
  ],
})
export class AppModule {}
