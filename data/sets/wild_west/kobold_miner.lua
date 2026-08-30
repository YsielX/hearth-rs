local card = {
    api_version = 1,
    id = "WW_001", rarity = "common",
    name = "Kobold Miner",
    text = "<b>Battlecry:</b> <b>Excavate</b>\na treasure.",
    set = "WILD_WEST",
    type = "minion",
    cost = 2,
    attack = 1,
    health = 1,
    keywords = { "battlecry", "excavate" },
}

local treasures = {
    [1] = { "WW_001t", "WW_001t2", "WW_001t3", "WW_001t4", "WW_001t18" },
    [2] = { "WW_001t5", "WW_001t7", "WW_001t8", "WW_001t9", "WW_001t16" },
    [3] = { "WW_001t11", "WW_001t12", "WW_001t13", "WW_001t14", "WW_001t17" },
    -- Neutral excavators cycle after the Epic tier; class-specific Legendary
    -- treasures are supplied by the eligible class cards themselves.
    [4] = { "WW_001t", "WW_001t2", "WW_001t3", "WW_001t4", "WW_001t18" },
}

function card.on_battlecry(ctx, self) end

function card.on_excavate(ctx, self, tier)
    ctx:random_value(treasures[tier], "receive_treasure")
end

function card.receive_treasure(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

card.tokens = {
    { id = "WW_001t", rarity = "common", name = "Rock", text = "Deal $3 damage.", set = "WILD_WEST", type = "spell", cost = 1, target_mode = "required", targets = function(ctx) return ctx:characters() end, on_play = function(ctx, self, target) cardlib.effects.damage(ctx, target, 3) end },
    { id = "WW_001t2", rarity = "common", name = "Water Source", text = "Restore #3 Health.\nDraw a card.", set = "WILD_WEST", type = "spell", cost = 1, target_mode = "required", targets = function(ctx) return ctx:characters() end, on_play = function(ctx, self, target) cardlib.effects.heal(ctx, target, 3); ctx:draw(ctx:controller(self), 1) end },
    { id = "WW_001t3", rarity = "common", name = "Fool's Azerite", text = "<b>Discover</b> a 2-Cost card.\nIt costs (0).", set = "WILD_WEST", type = "spell", cost = 1 },
    { id = "WW_001t4", rarity = "common", name = "Escaping Trogg", text = "<b>Rush</b>", set = "WILD_WEST", type = "minion", cost = 1, attack = 2, health = 3, keywords = { "rush" } },
    { id = "WW_001t18", rarity = "common", name = "Pouch of Coins", text = "Get two Coins.", set = "WILD_WEST", type = "spell", cost = 1, on_play = function(ctx, self) local p = ctx:controller(self); cardlib.effects.give_card(ctx, p, "GAME_005"); cardlib.effects.give_card(ctx, p, "GAME_005") end },
    { id = "WW_001t5", rarity = "rare", name = "Falling Stalactite", text = "Deal $3 damage\nto a minion and the\nenemy hero.", set = "WILD_WEST", type = "spell", cost = 2, target_mode = "required", targets = function(ctx) return ctx:minions() end, on_play = function(ctx, self, target) cardlib.effects.damage(ctx, target, 3); cardlib.effects.damage(ctx, ctx:player(ctx:opponent(ctx:controller(self))).hero, 3) end },
    { id = "WW_001t7", rarity = "rare", name = "Canary", text = "<b>Battlecry:</b> Return an\nenemy minion to its owner's hand.", set = "WILD_WEST", type = "minion", cost = 2, attack = 2, health = 2, tags = { "beast" }, keywords = { "battlecry" }, target_mode = "required_if_available", targets = function(ctx, self) return ctx:enemy_minions(self) end, on_battlecry = function(ctx, self, target) if target then ctx:move(target, "hand") end end },
    { id = "WW_001t8", rarity = "rare", name = "Glowing Glyph", text = "Give a minion and its neighbors +1/+2.", set = "WILD_WEST", type = "spell", cost = 2, target_mode = "required", targets = function(ctx) return ctx:minions() end, on_play = function(ctx, self, target) cardlib.effects.buff(ctx, target, 1, 2); for _, adjacent in ipairs(ctx:adjacent_minions(target)) do cardlib.effects.buff(ctx, adjacent, 1, 2) end end },
    { id = "WW_001t9", rarity = "rare", name = "Azerite Chunk", text = "<b>Discover</b> a 3-Cost card.\nIt costs (0).", set = "WILD_WEST", type = "spell", cost = 2 },
    { id = "WW_001t16", rarity = "rare", name = "Living Stone", text = "[x]<b>Taunt</b>\n<b>Deathrattle:</b> Summon a\nrandom 2-Cost minion.", set = "WILD_WEST", type = "minion", cost = 2, attack = 2, health = 4, tags = { "elemental" }, keywords = { "taunt" } },
    { id = "WW_001t11", rarity = "epic", name = "Ogrefist Boulder", text = "Set a minion's stats\nto 6/7.", set = "WILD_WEST", type = "location", cost = 3, health = 2, target_mode = "required", location_targets = function(ctx) return ctx:minions() end, on_location = function(ctx, self, target) local e = ctx:entity(target); cardlib.effects.buff(ctx, target, 6 - e.attack, 7 - e.max_health) end },
    { id = "WW_001t12", rarity = "epic", name = "Collapse!", text = "Deal $3 damage\nto all enemies.", set = "WILD_WEST", type = "spell", cost = 3, on_play = function(ctx, self) cardlib.effects.damage_all(ctx, ctx:enemy_characters(self), 3) end },
    { id = "WW_001t13", rarity = "epic", name = "Steelhide Mole", text = "<b>Taunt</b>\n<b>Reborn</b>\n<b>Elusive</b>", set = "WILD_WEST", type = "minion", cost = 3, attack = 3, health = 7, tags = { "beast" }, keywords = { "taunt", "reborn", "elusive" } },
    { id = "WW_001t14", rarity = "epic", name = "Azerite Gem", text = "<b>Discover</b> a 5-Cost card.\nIt costs (0).", set = "WILD_WEST", type = "spell", cost = 3 },
    { id = "WW_001t17", rarity = "epic", name = "Motherlode Drake", text = "<b>Rush</b>\n<b>Divine Shield</b>\n<b>Lifesteal</b>", set = "WILD_WEST", type = "minion", cost = 3, attack = 4, health = 3, tags = { "dragon" }, keywords = { "rush", "divine_shield", "lifesteal" } },
}

return card
