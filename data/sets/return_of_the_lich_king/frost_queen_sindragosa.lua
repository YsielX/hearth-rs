local wing = {
    name = "Sindragosa's Wing",
    text = "<b>Rush</b>\n<b>Freeze</b> any character damaged by this minion.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    class = "death_knight",
    collectible = false,
    cost = 1,
    attack = 2,
    health = 1,
    tags = { "undead", "dragon" },
    keywords = { "rush" },
    triggers = {{
        event = "damaged",
        timing = "after",
        active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.source == self and event.amount > 0
        end,
        effect = function(ctx, self, event) ctx:freeze(event.target) end,
    }},
}

local left_wing = {}
for key, value in pairs(wing) do left_wing[key] = value end
left_wing.id = "NX2_037t"

local right_wing = {}
for key, value in pairs(wing) do right_wing[key] = value end
right_wing.id = "NX2_037t2"

local card = {
    api_version = 1,
    id = "NX2_037",
    name = "Frost Queen Sindragosa",
    text = "<b>Colossal +2</b>\nAfter an enemy minion is <b>Frozen</b>, destroy it.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    class = "death_knight",
    rarity = "legendary",
    cost = 7,
    attack = 6,
    health = 6,
    rune_cost = { frost = 1 },
    tags = { "undead", "dragon" },
    keywords = { "colossal" },
    keyword_params = { colossal = 2 },
}

function card.on_colossal(ctx, self)
    local player = ctx:controller(self)
    local position = ctx:board_position(self)
    ctx:summon_at(player, "NX2_037t", position)
    ctx:summon_at(player, "NX2_037t2", position + 2)
end

card.triggers = {{
    event = "frozen",
    timing = "after",
    active_zones = { "board" },
    condition = function(ctx, self, event)
        local target = ctx:entity(event.target)
        return target.type == "minion"
            and target.zone == "board"
            and target.controller ~= ctx:controller(self)
    end,
    effect = function(ctx, self, event) cardlib.effects.destroy(ctx, event.target) end,
}}

card.tokens = { left_wing, right_wing }

return card
