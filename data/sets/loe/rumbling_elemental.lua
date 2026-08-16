local function has_battlecry(definition)
    for _, keyword in ipairs(definition.keywords or {}) do
        if keyword == "battlecry" then return true end
    end
    return false
end

local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "LOE_016",
    name = "Rumbling Elemental",
    text = "After you play a <b>Battlecry</b> minion, deal 2 damage to a random enemy.",
    set = "LOE",
    type = "minion",
    class = "shaman",
    rarity = "common",
    cost = 4,
    attack = 2,
    health = 6,
    tags = { "elemental" },
    triggers = {
        {
            event = "card_played",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                if event.player ~= ctx:controller(self) then return false end
                local played = ctx:entity(event.entity)
                return played.type == "minion"
                    and has_battlecry(ctx:card_definition(played.card_id))
            end,
            effect = function(ctx, self)
                local candidates = {}
                for _, enemy in ipairs(ctx:enemy_characters(self)) do
                    if not is_dormant(ctx, enemy) then
                        candidates[#candidates + 1] = enemy
                    end
                end
                if #candidates > 0 then
                    ctx:random_entity(candidates, "deal_rumbling_damage")
                end
            end,
        },
    },
}

function card.deal_rumbling_damage(ctx, self, target)
    cardlib.effects.damage(ctx, target, 2)
end

-- LOE_016t is the official child entity associated with this card.
card.tokens = {
    {
        id = "LOE_016t",
        name = "Rock",
        text = "<b>Taunt</b>",
        set = "LOE",
        type = "minion",
        cost = 1,
        attack = 0,
        health = 6,
        keywords = { "taunt" },
    },
}

return card
