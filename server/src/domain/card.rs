use serde::{Deserialize, Serialize};
use strum::EnumIter;
use valuable::{Valuable, Value};

use crate::domain::Color;
// 没有以下卡牌
// 兵营 = 10
// 密室 = 17
// 马厩 = 18
// 军械库 = 20
// 框架 = 22
// 雕塑 = 23
// 博物馆 = 25
// 贫民窟 = 26
// 宗教圣殿 = 27
// 纪念碑 = 28
// 采石场 = 30
// 神庙 = 31
// 地图室 = 32
// 象牙塔 = 33
// 工厂 = 34
// 许愿井 = 36
// 宝藏库 = 38
// 贼窝 = 41
// 金矿 = 44
// 公园 = 45

#[derive(Copy, Clone, EnumIter, Debug, PartialEq, Serialize, Deserialize)]
pub enum Card {
  // 绿色
  酒馆 = 2,
  贸易站 = 47,
  市场 = 5,
  码头 = 9,
  海港 = 12,
  市政厅 = 14,
  // 黄色
  庄园 = 8,
  城堡 = 11,
  皇宫 = 13,
  // 蓝色
  神殿 = 1,
  教堂 = 4,
  修道院 = 7,
  大教堂 = 15,
  // 红色
  瞭望台 = 3,
  监狱 = 6,
  战场 = 10, // 临时用兵营替代
  堡垒 = 16,
  // 紫色
  鬼城 = 19, // 卡牌上写的是鬼屋
  要塞 = 21,
  天文台 = 24,
  铁匠铺 = 37,
  实验室 = 35,
  墓地 = 29,
  魔法学院 = 40, // 卡牌上写的是魔法学校
  图书馆 = 46,
  城墙 = 43, // 卡牌上写的是长城
  龙门 = 42,
  大学 = 39,
}

impl Card {
  fn name(&self) -> &'static str {
    match self {
      Card::酒馆 => "酒馆",
      Card::贸易站 => "贸易站",
      Card::市场 => "市场",
      Card::码头 => "码头",
      Card::海港 => "海港",
      Card::市政厅 => "市政厅",
      Card::庄园 => "庄园",
      Card::城堡 => "城堡",
      Card::皇宫 => "皇宫",
      Card::神殿 => "神殿",
      Card::教堂 => "教堂",
      Card::修道院 => "修道院",
      Card::大教堂 => "大教堂",
      Card::瞭望台 => "瞭望台",
      Card::监狱 => "监狱",
      Card::战场 => "战场",
      Card::堡垒 => "堡垒",
      Card::鬼城 => "鬼城",
      Card::要塞 => "要塞",
      Card::天文台 => "天文台",
      Card::铁匠铺 => "铁匠铺",
      Card::实验室 => "实验室",
      Card::墓地 => "墓地",
      Card::魔法学院 => "魔法学院",
      Card::图书馆 => "图书馆",
      Card::城墙 => "城墙",
      Card::龙门 => "龙门",
      Card::大学 => "大学",
    }
  }

  fn color_fee_number(&self) -> (Color, u32, u32) {
    match self {
      // 绿色
      Card::酒馆 => (Color::绿, 1, 5),
      Card::贸易站 => (Color::绿, 2, 3),
      Card::市场 => (Color::绿, 2, 4),
      Card::码头 => (Color::绿, 3, 3),
      Card::海港 => (Color::绿, 4, 3),
      Card::市政厅 => (Color::绿, 5, 2),
      // 黄色
      Card::庄园 => (Color::黄, 3, 5),
      Card::城堡 => (Color::黄, 4, 4),
      Card::皇宫 => (Color::黄, 5, 3),
      // 蓝色
      Card::神殿 => (Color::蓝, 1, 3),
      Card::教堂 => (Color::蓝, 2, 3),
      Card::修道院 => (Color::蓝, 3, 3),
      Card::大教堂 => (Color::蓝, 5, 2),
      // 红色
      Card::瞭望台 => (Color::红, 1, 3),
      Card::监狱 => (Color::红, 2, 3),
      Card::战场 => (Color::红, 3, 3),
      Card::堡垒 => (Color::红, 5, 2),
      // 紫色
      Card::鬼城 => (Color::紫, 2, 1),
      Card::要塞 => (Color::紫, 3, 2),
      Card::天文台 => (Color::紫, 5, 1),
      Card::铁匠铺 => (Color::紫, 5, 1),
      Card::实验室 => (Color::紫, 5, 1),
      Card::墓地 => (Color::紫, 5, 1),
      Card::魔法学院 => (Color::紫, 6, 1),
      Card::图书馆 => (Color::紫, 6, 1),
      Card::城墙 => (Color::紫, 6, 1),
      Card::龙门 => (Color::紫, 6, 1),
      Card::大学 => (Color::紫, 6, 1),
    }
  }

  pub fn color(&self) -> Color {
    self.color_fee_number().0
  }

  pub fn fee(&self) -> u32 {
    self.color_fee_number().1
  }

  pub fn number(&self) -> u32 {
    self.color_fee_number().2
  }

  pub fn score(&self) -> u32 {
    match self {
      Card::龙门 => 2 + self.fee(),
      Card::大学 => 2 + self.fee(),
      _ => self.fee(),
    }
  }
}

impl Valuable for Card {
  fn as_value(&self) -> Value<'_> {
    Value::String(self.name())
  }

  fn visit(&self, visit: &mut dyn valuable::Visit) {
    visit.visit_value(Value::String(self.name()));
  }
}
