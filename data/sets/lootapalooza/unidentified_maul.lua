local outcomes = { "LOOT_286t1", "LOOT_286t2", "LOOT_286t3", "LOOT_286t4" }

local function reveal(ctx, self) ctx:random_value(outcomes, "reveal_maul") end

local card = {
    api_version = 1, id = "LOOT_286", name = "Unidentified Maul",
    text = "Gains a bonus effect in your hand.", set = "LOOTAPALOOZA", type = "weapon",
    class = "paladin", rarity = "rare", cost = 3, attack = 2, health = 2,
    triggers = {
        { event = "game_started", timing = "after", active_zones = { "hand" }, effect = reveal },
        { event = "card_drawn", timing = "after", active_zones = { "hand" }, condition = function(ctx, self, event) return event.entity == self end, effect = reveal },
        { event = "card_created", timing = "after", active_zones = { "hand" }, condition = function(ctx, self, event) return event.entity == self end, effect = reveal },
    },
}
function card.reveal_maul(ctx, self, id) ctx:transform(self, id) end

local function weapon(id, name, text, battlecry)
    return {
        id = id, name = name, text = text, set = "LOOTAPALOOZA", type = "weapon",
        class = "paladin", collectible = false, cost = 3, attack = 2, health = 2,
        keywords = { "battlecry" }, on_battlecry = battlecry,
    }
end

card.tokens = {
    weapon("LOOT_286t1", "Champion's Maul", "<b>Battlecry:</b> Summon two 1/1 Silver Hand Recruits.", function(ctx, self)
        local player = ctx:controller(self); ctx:summon(player, "CS2_101t"); ctx:summon(player, "CS2_101t")
    end),
    weapon("LOOT_286t2", "Sacred Maul", "<b>Battlecry:</b> Give your minions <b>Taunt</b>.", function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do ctx:grant_keyword(minion, "taunt") end
    end),
    weapon("LOOT_286t3", "Blessed Maul", "<b>Battlecry:</b> Give your minions +1 Attack.", function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do ctx:buff(minion, 1, 0) end
    end),
    weapon("LOOT_286t4", "Purifier's Maul", "<b>Battlecry:</b> Give your minions <b>Divine Shield</b>.", function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do ctx:grant_keyword(minion, "divine_shield") end
    end),
}
return card
