import {
  BadRequestException,
  Body,
  Controller,
  Delete,
  Get,
  HttpStatus,
  NotFoundException,
  Param,
  Post,
  Request,
  Res,
  UseFilters,
} from "@nestjs/common";
import { Types } from "mongoose";
import { Public } from "../common/decorator/auth.decorator";
import { Roles } from "../common/decorator/roles.decorator";
import { Role } from "../common/enums/role.enum";
import { MongoExceptionFilter } from "../common/filter/mongo-exception.filter";
import type { CreateUserDto } from "./dto/create-user.dto";
import type { UpdateUserDto } from "./dto/update-user.dto";
import type { UsersService } from "./users.service";

@Controller({
  path: "users",
  version: "1",
})
export class UsersController {
  constructor(private usersService: UsersService) {}

  @Get()
  @Roles(Role.Admin)
  async findAll() {
    return this.usersService.findAll();
  }

  @Get(":id")
  async findOne(@Param("id") id: string) {
    if (!Types.ObjectId.isValid(id)) {
      throw new BadRequestException();
    }

    const user = await this.usersService.findOne({ _id: id });

    return user;
  }

  @Post("profile/:id")
  @Roles(Role.User)
  async updateProfile(
    @Param("id") id: string,
    @Body() updateUserDto: UpdateUserDto,
  ) {
    if (!Types.ObjectId.isValid(id)) {
      throw new BadRequestException();
    }

    const { firstName, lastName, gender, age, avatar } = updateUserDto;
    const user = await this.usersService.findOneAndUpdate(
      { _id: id },
      {
        firstName,
        lastName,
        gender,
        age,
        avatar,
      },
    );

    if (!user) {
      throw new NotFoundException("User does not exist.");
    }

    return user;
  }

  @Public()
  @Post()
  @UseFilters(MongoExceptionFilter)
  async create(@Body() createUserDto: CreateUserDto) {
    const user = await this.usersService.create(createUserDto);

    if (user) {
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const { password, ...result } = user;

      return result;
    }

    return user;
  }

  @Delete(":id")
  @Roles(Role.Admin)
  async deleteOne(@Res() res, @Param("id") id: string) {
    if (!Types.ObjectId.isValid(id)) {
      throw new BadRequestException();
    }

    const user = await this.usersService.deleteOne(id);

    return res.status(HttpStatus.OK).json({
      message: "User has been deleted.",
      user,
    });
  }

  @Delete()
  @Roles(Role.Admin)
  async deleteAll() {
    return this.usersService.deleteAll();
  }

  @Post("password")
  @Roles(Role.User)
  @UseFilters(MongoExceptionFilter)
  async updatePassword(@Request() req, @Body() body) {
    const userId = req.user?.userId;

    if (!Types.ObjectId.isValid(userId)) {
      throw new BadRequestException();
    }

    const { password, changePassword } = body;

    if (password?.length < 6) {
      throw new BadRequestException(
        "Password must be at least 6 characters long.",
      );
    } else if (password !== changePassword) {
      throw new BadRequestException("Passwords do not match.");
    }

    const msg = await this.usersService.updatePassword(userId, password);

    return msg;
  }
}
