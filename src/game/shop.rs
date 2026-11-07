//! The shop between sectors.

use super::*;

impl Game {
    /// Development helper: open the shop with some coins.
    pub fn debug_shop(&mut self) {
        self.coins = 3000.0;
        self.shop_sel = 0;
        self.roll_offers();
        self.phase = Phase::Shop;
    }

    /// The whole catalogue: everything the shop sells, from the cheapest to the most
    /// expensive. The strongest weapons are only found here.
    pub(super) fn roll_offers(&mut self) {
        let mut items: Vec<Item> = vec![
            Item::ExtraLife,
            Item::Bomb,
            Item::Shield,
            Item::Upgrade,
            Item::Thruster,
            Item::Trigger,
            Item::Magnet,
            Item::Insurance,
            Item::EnergyCell,
            Item::Amulet,
            Item::Purse,
            Item::MysteryCard,
            Item::SideCannons,
            Item::LaserSight,
            Item::Capacitor,
            Item::MysteryBox,
            Item::Autofire,
            Item::DamageCore,
        ];
        if self.cfg.rank >= 5 {
            items.push(Item::Drone);
        }
        if self.cfg.rank >= 11 {
            items.extend([Item::Bank, Item::Radar, Item::TreasureMap, Item::Mercenary]);
        }
        if self.can_evolve() {
            items.push(Item::Evolve(self.player.weapon));
        }
        for w in self.weapon_pool() {
            if w != self.player.weapon && !w.is_relic() {
                items.push(Item::Weapon(w));
            }
        }
        items.sort_by_key(|it| (self.price(*it), it.key()));
        self.shop_offers = items;
    }

    pub(super) fn bought(&self, it: Item) -> u32 {
        self.shop_buys
            .iter()
            .find(|(k, _)| *k == it.key())
            .map(|(_, n)| *n)
            .unwrap_or(0)
    }

    pub(super) fn price(&self, it: Item) -> u32 {
        let mut p = it.price() as f32 * (1.0 + 0.75 * self.bought(it) as f32) * self.mods.shop_mul;
        if self.grudge {
            p *= 1.3;
        }
        if self.cfg.daily == Some(Daily::DoubleCoins) {
            p *= 1.5;
        }
        p.round() as u32
    }

    pub(super) fn shop_block(&self, it: Item) -> Option<&'static str> {
        let p = &self.player;
        match it {
            Item::ExtraLife if p.lives >= MAX_LIVES => Some("MAX"),
            Item::Bomb if p.bombs >= 5 => Some("MAX"),
            Item::Upgrade if p.level >= 5 => Some("MAX"),
            Item::Shield if p.shield => Some("OWNED"),
            Item::Thruster if p.speed_lvl >= 3 => Some("MAX"),
            Item::Trigger if p.fire_lvl >= MAX_FIRE_LVL => Some("MAX"),
            Item::Magnet if p.magnet_lvl >= 3 || self.mods.magnet_all => Some("MAX"),
            Item::Drone if self.drones_total() >= 3 => Some("MAX"),
            Item::SideCannons if self.side_cannons => Some("OWNED"),
            Item::Insurance if self.insurance || self.mods.keep_level => Some("OWNED"),
            Item::Bank if self.bank => Some("OWNED"),
            Item::Purse if self.purse => Some("OWNED"),
            Item::Amulet if self.bought(Item::Amulet) >= 2 => Some("MAX"),
            Item::Radar if self.bought(Item::Radar) >= 1 => Some("OWNED"),
            Item::TreasureMap if self.treasure_map => Some("OWNED"),
            Item::Mercenary if self.ally_sectors > 0 => Some("OWNED"),
            Item::Weapon(w) if w == p.weapon => Some("EQUIPPED"),
            Item::Autofire if self.arcade.autofire => Some("OWNED"),
            Item::LaserSight if self.arcade.sight => Some("OWNED"),
            Item::Capacitor if self.arcade.capacitor => Some("OWNED"),
            Item::DamageCore if self.arcade.core >= 6 => Some("MAX"),
            Item::Evolve(w) if self.player.evolved_weapon == Some(w) => Some("OWNED"),
            Item::Evolve(w) if w != self.player.weapon || self.player.level < 5 => Some("LV 5"),
            _ => None,
        }
    }

    pub(super) fn update_shop(&mut self, inp: &Input, au: &mut Audio) -> Phase {
        // the catalogue plus the DEPART row
        let n = self.shop_offers.len() + 1;
        if let Some(h) = inp.hover
            && h < n
            && h != self.shop_sel
        {
            self.shop_sel = h;
            au.play_vol(Sfx::MenuMove, 0.5);
        }
        // Mouse: a click on the selected row buys it.
        let mouse_buy = inp.select_click == Some(self.shop_sel);
        if inp.up_p {
            self.shop_sel = (self.shop_sel + n - 1) % n;
            au.play(Sfx::MenuMove);
        }
        if inp.down_p {
            self.shop_sel = (self.shop_sel + 1) % n;
            au.play(Sfx::MenuMove);
        }
        if inp.back {
            self.shop_sel = n - 1;
            au.play(Sfx::MenuMove);
        }
        if inp.confirm || mouse_buy {
            let i = self.shop_sel;
            if i == n - 1 {
                au.play(Sfx::MenuOk);
                return self.leave_shop();
            }
            let it = self.shop_offers[i];
            let price = self.price(it);
            if self.shop_block(it).is_some() || (self.coins.floor() as u32) < price {
                au.play(Sfx::Deny);
            } else {
                self.coins -= price as f32;
                let key = it.key();
                match self.shop_buys.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, n)) => *n += 1,
                    None => self.shop_buys.push((key, 1)),
                }
                au.play(Sfx::Buy);
                let p = &mut self.player;
                match it {
                    Item::ExtraLife => p.lives += 1,
                    Item::Bomb => p.bombs += 1,
                    Item::Shield => p.shield = true,
                    Item::Upgrade => p.level += 1,
                    Item::Weapon(w) => encounters::pick_new_weapon(p, w),
                    Item::Thruster => p.speed_lvl += 1,
                    Item::Trigger => p.fire_lvl += 1,
                    Item::Magnet => p.magnet_lvl += 1,
                    Item::Drone => self.drones_bought += 1,
                    Item::SideCannons => self.side_cannons = true,
                    Item::Insurance => self.insurance = true,
                    Item::EnergyCell => {
                        p.meter = 1.0;
                        self.mods.meter_mul += 0.2;
                    }
                    Item::Bank => self.bank = true,
                    Item::Radar => self.mods.card_mul += 0.6,
                    Item::Amulet => self.mods.powerup_mul += 0.3,
                    Item::Purse => self.purse = true,
                    Item::TreasureMap => self.treasure_map = true,
                    Item::Mercenary => self.ally_sectors = 2,
                    Item::MysteryCard => {
                        let c = cards::roll(0);
                        self.give_card(c, au);
                    }
                    Item::Autofire => self.arcade.autofire = true,
                    Item::LaserSight => self.arcade.sight = true,
                    Item::Capacitor => self.arcade.capacitor = true,
                    Item::MysteryBox => self.open_box(au),
                    Item::DamageCore => self.arcade.core += 1,
                    Item::Evolve(_) => self.evolve(au),
                }
                self.mirror_purchase(it);
                self.check_mastery();
                // prices and availability changed: rebuild the list, keep the cursor on the item
                let key = it.key();
                self.roll_offers();
                self.shop_sel = self
                    .shop_offers
                    .iter()
                    .position(|o| o.key() == key)
                    .unwrap_or(self.shop_sel.min(self.shop_offers.len()));
            }
        }
        Phase::Shop
    }

    pub(super) fn leave_shop(&mut self) -> Phase {
        self.rocks.clear();
        self.wave += 1;
        if self.cfg.mode == Mode::Campaign
            && self.cycle == 0
            && self.sector() == 4
            && self.deaths == 0
            && !self.secret_done
        {
            return Phase::Portal { t: 0.0 };
        }
        Phase::Intro { t: 0.0 }
    }

    pub(super) fn draw_shop(&self, sp: &Sprites) {
        draw_rectangle(0.0, 0.0, W, H, Color::new(0.0, 0.0, 0.05, 0.85));
        ui::text_outline(
            "SHOP",
            W / 2.0,
            60.0,
            44.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
        );
        let sub = if self.grudge {
            "The Merchant glares at you: prices +30%"
        } else {
            "Spend your coins before the next sector"
        };
        ui::text_c(
            sub,
            W / 2.0,
            84.0,
            14.0,
            if self.grudge { RED } else { DIM },
        );
        coins::draw_coin(vec2(W / 2.0 - 40.0, 108.0), 100, self.time, 1.4);
        ui::text(
            &format!("{}", self.coins.floor() as u32),
            W / 2.0 - 24.0,
            116.0,
            26.0,
            CYAN,
        );

        let y0 = 134.0;
        let row_h = 46.0;
        let n = self.shop_offers.len();
        let visible = 10usize;
        // keep the selection in view
        let first = self
            .shop_sel
            .saturating_sub(visible / 2)
            .min(n.saturating_sub(visible));
        for (row, i) in (first..(first + visible).min(n)).enumerate() {
            let it = &self.shop_offers[i];
            let y = y0 + row as f32 * row_h;
            let sel = self.shop_sel == i;
            let blocked = self.shop_block(*it);
            let price = self.price(*it);
            let afford = self.coins.floor() as u32 >= price;
            let bg = if sel {
                Color::new(0.3, 0.25, 0.05, 0.6)
            } else {
                Color::new(0.1, 0.1, 0.2, 0.5)
            };
            draw_rectangle(20.0, y, W - 40.0, row_h - 5.0, bg);
            ui::hot(
                Rect::new(20.0, y, W - 40.0, row_h - 5.0),
                ui::Hot::Select(i),
            );
            if sel {
                draw_rectangle_lines(20.0, y, W - 40.0, row_h - 5.0, 2.0, GOLD);
            }
            match it {
                Item::Weapon(w) => crate::icons::weapon_badge(*w, vec2(44.0, y + 20.0), 14.0, 1.0),
                _ => sp
                    .icon(it.icon())
                    .draw(vec2(44.0, y + 20.0), 2.6, 0.0, WHITE, false),
            }
            let name_c = if blocked.is_some() { DIM } else { WHITE };
            ui::text(&it.name(), 68.0, y + 18.0, 17.0, name_c);
            ui::text(it.desc(), 68.0, y + 35.0, 12.0, DIM);
            match blocked {
                Some(why) => ui::text_r(why, W - 32.0, y + 27.0, 15.0, DIM),
                None => ui::text_r(
                    &format!("{price}"),
                    W - 32.0,
                    y + 27.0,
                    21.0,
                    if afford { CYAN } else { RED },
                ),
            }
        }
        // scroll marks
        let arrow = Color::new(1.0, 0.85, 0.4, 0.5 + (self.time * 4.0).sin().abs() * 0.5);
        if first > 0 {
            ui::text_c("^ MORE ^", W / 2.0, y0 - 2.0, 11.0, arrow);
        }
        if first + visible < n {
            ui::text_c(
                "v MORE v",
                W / 2.0,
                y0 + visible as f32 * row_h + 4.0,
                11.0,
                arrow,
            );
        }
        let y = y0 + visible as f32 * row_h + 22.0;
        ui::hot(Rect::new(40.0, y - 6.0, W - 80.0, 34.0), ui::Hot::Select(n));
        let next = match self.cfg.mode {
            Mode::BossRush => format!("FACE THE BOSS {}", self.wave + 1),
            _ => format!("DEPART FOR SECTOR {}", self.absolute_sector() + 1),
        };
        if self.shop_sel == n {
            ui::text_c(&format!("> {next} <"), W / 2.0, y + 18.0, 22.0, GOLD);
        } else {
            ui::text_c(&next, W / 2.0, y + 18.0, 20.0, DIM);
        }
        let p = &self.player;
        let status = format!(
            "SHIPS {}  BOMBS {}  {} LV{}  DRONES {}",
            p.lives,
            p.bombs,
            p.weapon.name(),
            p.level,
            self.drones_total()
        );
        ui::text_c(&status, W / 2.0, H - 30.0, 13.0, DIM);
        let power = format!(
            "FIREPOWER x{:.2}   (weapon {} x{:.2} · cores {} x{:.2})",
            self.power_mul(),
            "*".repeat(p.weapon.tier() as usize),
            p.weapon.tier_mul(),
            self.arcade.core,
            self.core_mul()
        );
        ui::text_c(&power, W / 2.0, H - 12.0, 13.0, GOLD);
    }
}
