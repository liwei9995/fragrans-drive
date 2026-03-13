import { Controller, Get, Post, Request, UseGuards } from "@nestjs/common";
import type { AppService } from "./app.service";
import type { AuthService } from "./auth/auth.service";
import { JwtAuthGuard } from "./auth/jwt-auth.guard";
import { LocalAuthGuard } from "./auth/local-auth.guard";
import { Public } from "./common/decorator/auth.decorator";

@Controller({
  version: "1",
})
export class AppController {
  constructor(
    readonly _appService: AppService,
    private readonly authService: AuthService,
  ) {}

  @Public()
  @UseGuards(LocalAuthGuard)
  @Post("auth/login")
  async login(@Request() req) {
    return this.authService.login(req);
  }

  @UseGuards(JwtAuthGuard)
  @Get("profile")
  getProfile(@Request() req) {
    return req.user;
  }
}
