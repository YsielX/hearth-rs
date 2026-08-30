local card = {
    api_version = 1,
    id = "RLK_512",
    name = "Glacial Advance",
    text = "Deal $4 damage.\nYour next spell this turn costs (2) less.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "common",
    spell_school = "frost",
    cost = 3,
    rune_cost = { frost = 1 },
    target_mode = "required",
    targets = function(ctx) return ctx:characters() end,
}

function card.on_play(ctx, self, target)
    cardlib.effects.damage(ctx, target, 4)
    ctx:set_data(self, "glacial_advance_active", 1)
end

card.auras = {{
    active_zones = { "graveyard" },
    cost = -2,
    targets = function(ctx, self)
        local targets = {}
        if ctx:get_data(self, "glacial_advance_active") == 0 then return targets end
        for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
            if ctx:entity(entity).type == "spell" then targets[#targets + 1] = entity end
        end
        return targets
    end,
}}

local function consume(ctx, self, event)
    return ctx:get_data(self, "glacial_advance_active") == 1
        and event.player == ctx:controller(self)
        and ctx:entity(event.entity).type == "spell"
        and event.entity ~= self
end

card.triggers = {
    {
        event = "spell_cast", timing = "after", active_zones = { "graveyard" },
        condition = function(ctx, self, event) return event.player_cast and consume(ctx, self, event) end,
        effect = function(ctx, self) ctx:set_data(self, "glacial_advance_active", 0) end,
    },
    {
        event = "card_countered", timing = "after", active_zones = { "graveyard" },
        condition = consume,
        effect = function(ctx, self) ctx:set_data(self, "glacial_advance_active", 0) end,
    },
    {
        event = "turn_ended", timing = "after", active_zones = { "graveyard" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
                and ctx:get_data(self, "glacial_advance_active") == 1
        end,
        effect = function(ctx, self) ctx:set_data(self, "glacial_advance_active", 0) end,
    },
}

return card
