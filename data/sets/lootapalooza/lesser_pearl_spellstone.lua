local function healing_trigger(next_id)
    return {{
        event = "healed", timing = "after", active_zones = { "hand" },
        condition = function(ctx, self, event)
            return event.amount > 0 and ctx:controller(event.source) == ctx:controller(self)
        end,
        effect = function(ctx, self, event)
            local restored = ctx:get_data(self, "pearl_restored") + event.amount
            ctx:set_data(self, "pearl_restored", restored)
            if restored >= 3 then ctx:transform(self, next_id) end
        end,
    }}
end

local function spellstone(id, name, text, token, next_id)
    local result = {
        id = id, name = name, text = text, set = "LOOTAPALOOZA", type = "spell",
        class = "paladin", collectible = false, spell_school = "holy", cost = 2,
        on_play = function(ctx, self) ctx:summon(ctx:controller(self), token) end,
    }
    if next_id then result.triggers = healing_trigger(next_id) end
    return result
end

local card = spellstone("LOOT_091", "Lesser Pearl Spellstone",
    "Summon a 2/2 Spirit with <b>Taunt</b>. <i>(Restore 3 Health to upgrade.)</i>",
    "LOOT_091t", "LOOT_091t1")
card.api_version = 1
card.collectible = true
card.rarity = "rare"
card.tokens = {
    spellstone("LOOT_091t1", "Pearl Spellstone",
        "Summon a 4/4 Spirit with <b>Taunt</b>.", "LOOT_091t1t", "LOOT_091t2"),
    spellstone("LOOT_091t2", "Greater Pearl Spellstone",
        "Summon a 6/6 Spirit with <b>Taunt</b>.", "LOOT_091t2t", nil),
    { id = "LOOT_091t", name = "Guardian Spirit", text = "<b>Taunt</b>", set = "LOOTAPALOOZA", type = "minion", class = "paladin", collectible = false, cost = 2, attack = 2, health = 2, tags = { "undead" }, keywords = { "taunt" } },
    { id = "LOOT_091t1t", name = "Guardian Spirit", text = "<b>Taunt</b>", set = "LOOTAPALOOZA", type = "minion", class = "paladin", collectible = false, cost = 4, attack = 4, health = 4, tags = { "undead" }, keywords = { "taunt" } },
    { id = "LOOT_091t2t", name = "Guardian Spirit", text = "<b>Taunt</b>", set = "LOOTAPALOOZA", type = "minion", class = "paladin", collectible = false, cost = 6, attack = 6, health = 6, tags = { "undead" }, keywords = { "taunt" } },
}
return card
