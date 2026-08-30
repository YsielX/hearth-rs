local upgrades = {
    warrior = "HERO_01bp2", shaman = "HERO_02bp2", rogue = "HERO_03bp2",
    paladin = "HERO_04bp2", hunter = "HERO_05bp2", druid = "HERO_06bp2",
    warlock = "HERO_07bp2", mage = "HERO_08bp2", priest = "HERO_09bp2",
    demon_hunter = "HERO_10bp2",
}

local card = {
    api_version = 1, id = "AT_132", name = "Justicar Trueheart",
    text = "<b>Battlecry:</b> Replace your starting Hero Power with a better one.", set = "TGT",
    type = "minion", rarity = "legendary", cost = 5, attack = 6, health = 4,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local upgraded = upgrades[ctx:player(player).class]
    if upgraded ~= nil then ctx:replace_hero_power(player, upgraded) end
end

card.tokens = {
    {
        id = "HERO_01bp2", name = "Tank Up!", text = "<b>Hero Power</b>\nGain $d4 Armor.",
        set = "LEGACY", type = "hero_power", class = "warrior", cost = 2,
        on_play = function(ctx, self) ctx:gain_armor(ctx:controller(self), 4) end,
    },
    {
        id = "HERO_02bp2", name = "Totemic Slam",
        text = "<b>Hero Power</b>\nSummon a Totem of your choice.",
        set = "LEGACY", type = "hero_power", class = "shaman", cost = 2,
        on_play = function(ctx, self)
            local player = ctx:controller(self)
            local present = {}
            for _, entity in ipairs(ctx:board(player)) do present[ctx:entity(entity).card_id] = true end
            local options = {}
            for _, option in ipairs({
                { id = "AT_132_SHAMANa", name = "Healing Totem" },
                { id = "AT_132_SHAMANb", name = "Searing Totem" },
                { id = "AT_132_SHAMANc", name = "Stoneclaw Totem" },
                { id = "AT_132_SHAMANd", name = "Wrath of Air Totem" },
            }) do
                if not present[option.id] then
                    options[#options + 1] = { label = option.name, value = option.id }
                end
            end
            if #options > 0 then ctx:choose_options(player, "Choose a Totem", options, "summon_chosen_totem") end
        end,
        summon_chosen_totem = function(ctx, self, card_id)
            ctx:summon(ctx:controller(self), card_id)
        end,
    },
    {
        id = "AT_132_SHAMANa", name = "Healing Totem",
        text = "At the end of your turn, restore #1 Health to all friendly minions.",
        set = "TGT", type = "minion", class = "shaman",
        cost = 1, attack = 0, health = 2, tags = { "totem" },
        triggers = {{
            event = "turn_ended", timing = "after", active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                cardlib.effects.heal_all(ctx, ctx:friendly_minions(self), 1)
            end,
        }},
    },
    {
        id = "AT_132_SHAMANb", name = "Searing Totem", text = "", set = "TGT",
        type = "minion", class = "shaman", cost = 1, attack = 1, health = 1,
        tags = { "totem" },
    },
    {
        id = "AT_132_SHAMANc", name = "Stoneclaw Totem", text = "<b>Taunt</b>",
        set = "TGT", type = "minion", class = "shaman", cost = 1,
        attack = 0, health = 2, tags = { "totem" }, keywords = { "taunt" },
    },
    {
        id = "AT_132_SHAMANd", name = "Wrath of Air Totem",
        text = "<b>Spell Damage +1</b>", set = "TGT", type = "minion",
        class = "shaman", cost = 1, attack = 0, health = 2, tags = { "totem" },
        keywords = { "spell_damage" }, keyword_params = { spell_damage = 1 },
    },
    {
        id = "HERO_03bp2", name = "Poisoned Daggers",
        text = "<b>Hero Power</b>\nEquip a 2/2 Weapon.", set = "LEGACY",
        type = "hero_power", class = "rogue", cost = 2,
        on_play = function(ctx, self) ctx:equip_weapon(ctx:controller(self), "AT_132_ROGUEt") end,
    },
    {
        id = "AT_132_ROGUEt", name = "Poisoned Dagger", text = "", set = "TGT",
        type = "weapon", class = "rogue", cost = 1, attack = 2, health = 2,
    },
    {
        id = "HERO_04bp2", name = "The Silver Hand",
        text = "<b>Hero Power</b>\nSummon two {0} Recruits.", set = "LEGACY",
        type = "hero_power", class = "paladin", cost = 2,
        on_play = function(ctx, self)
            local player = ctx:controller(self)
            ctx:summon(player, "CS2_101t")
            ctx:summon(player, "CS2_101t")
        end,
    },
    {
        id = "HERO_05bp2", name = "Ballista Shot",
        text = "<b>Hero Power</b>\nDeal $3 damage to the enemy hero.", set = "LEGACY",
        type = "hero_power", class = "hunter", cost = 2,
        on_play = function(ctx, self)
            local enemy = ctx:opponent(ctx:controller(self))
            cardlib.effects.damage(ctx, ctx:player(enemy).hero, 3)
        end,
    },
    {
        id = "HERO_06bp2", name = "Dire Shapeshift",
        text = "<b>Hero Power</b>\n+$a2 Attack this turn.\n+$d2 Armor.", set = "LEGACY",
        type = "hero_power", class = "druid", cost = 2,
        on_play = function(ctx, self)
            local player = ctx:controller(self)
            cardlib.effects.buff_until_end_of_turn(ctx, ctx:player(player).hero, 2, 0)
            ctx:gain_armor(player, 2)
        end,
    },
    {
        id = "HERO_07bp2", name = "Soul Tap", text = "<b>Hero Power</b>\nDraw a card.",
        set = "LEGACY", type = "hero_power", class = "warlock", cost = 2,
        on_play = function(ctx, self) ctx:draw(ctx:controller(self), 1) end,
    },
    {
        id = "HERO_08bp2", name = "Fireblast Rank 2",
        text = "<b>Hero Power</b>\nDeal $2 damage.", set = "LEGACY",
        type = "hero_power", class = "mage", cost = 2, target_mode = "required",
        targets = function(ctx) return ctx:characters() end,
        on_play = function(ctx, self, target) cardlib.effects.damage(ctx, target, 2) end,
    },
    {
        id = "HERO_09bp2", name = "Heal", text = "<b>Hero Power</b>\nRestore #4 Health.",
        set = "LEGACY", type = "hero_power", class = "priest", cost = 2,
        target_mode = "required", targets = function(ctx) return ctx:characters() end,
        on_play = function(ctx, self, target) cardlib.effects.heal(ctx, target, 4) end,
    },
    {
        id = "HERO_10bp2", name = "Demon's Bite",
        text = "[x]<b>Hero Power</b>\n+$a2 Attack this turn.", set = "LEGACY",
        type = "hero_power", class = "demon_hunter", cost = 1,
        on_play = function(ctx, self)
            local player = ctx:controller(self)
            cardlib.effects.buff_until_end_of_turn(ctx, ctx:player(player).hero, 2, 0)
        end,
    },
}

return card
