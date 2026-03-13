import "dotenv/config";
import { Logger, ValidationPipe, VersioningType } from "@nestjs/common";
import { NestFactory } from "@nestjs/core";
import { DocumentBuilder, SwaggerModule } from "@nestjs/swagger";
import * as config from "config";
import helmet from "helmet";
import { AppModule } from "./app.module";

const port =
  Number(process.env.HTTP_PORT) || config.get<number>("http.port") || 3847;
const logger = new Logger("NestApplicationMain");

async function bootstrap() {
  const app = await NestFactory.create(AppModule);

  app.enableShutdownHooks();
  app.useGlobalPipes(new ValidationPipe());
  app.enableVersioning({ type: VersioningType.URI });

  app.use(helmet({ crossOriginResourcePolicy: false }));
  const corsOrigins = process.env.CORS_ORIGINS?.split(",")
    .map((o) => o.trim())
    .filter(Boolean);
  app.enableCors({
    origin: corsOrigins?.length ? corsOrigins : "*",
    credentials: !!corsOrigins?.length,
  });

  const swaggerConfig = new DocumentBuilder()
    .setTitle("Fragrans API")
    .setDescription("自托管文件存储服务 API")
    .setVersion("1.0")
    .addBearerAuth()
    .build();
  const document = SwaggerModule.createDocument(app, swaggerConfig);
  SwaggerModule.setup("api", app, document);

  await app.listen(port);

  logger.log(`Application is running on: ${await app.getUrl()}`);
  logger.log(`Swagger docs: ${await app.getUrl()}/api`);
}

void bootstrap();
