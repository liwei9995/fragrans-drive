import { Injectable } from "@nestjs/common";
import type { JwtService } from "@nestjs/jwt";
import * as bcrypt from "bcryptjs";
import { Public } from "../common/decorator/auth.decorator";
import type { UsersService } from "../users/users.service";

@Injectable()
export class AuthService {
  constructor(
    private usersService: UsersService,
    private jwtService: JwtService,
  ) {}

  async validateUser(username: string, pass: string): Promise<any> {
    const user = await this.usersService.findOne(
      { email: username },
      "+password",
    );

    if (user && (await bcrypt.compare(pass, user.password))) {
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { password, ...result } = user;

      return result;
    }

    return null;
  }

  @Public()
  async login(req: any) {
    const user = req?.user;
    const payload = {
      sub: user?._id,
      email: user?.email,
      firstName: user?.firstName,
      lastName: user?.lastName,
      gender: user?.gender,
      age: user?.age,
      avatar: user?.avatar,
      roles: user?.roles,
    };

    return {
      access_token: this.jwtService.sign(payload),
    };
  }
}
