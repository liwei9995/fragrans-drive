import {
  type CallHandler,
  type ExecutionContext,
  Injectable,
  Logger,
  type NestInterceptor,
} from "@nestjs/common";
import type { Observable } from "rxjs";
import { tap } from "rxjs/operators";

@Injectable()
export class LoggingInterceptor implements NestInterceptor {
  private readonly logger = new Logger(LoggingInterceptor.name);

  intercept(context: ExecutionContext, next: CallHandler): Observable<any> {
    const [req] = context.getArgs();
    const methodKey = context.getHandler().name;
    const className = context.getClass().name;
    const now = Date.now();

    return next.handle().pipe(
      tap(() => {
        const queue = [];

        if (req) {
          queue.push(req.method);
          queue.push(req.url);
        }
        methodKey && queue.push(methodKey);
        className && queue.push(className);
        queue.push(`${Date.now() - now}ms`);
        this.logger.log(queue.join(" - "));
      }),
    );
  }
}
