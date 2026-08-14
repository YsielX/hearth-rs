local function has_dragon_tag(ctx, entity)
    for _, tag in ipairs(ctx:entity(entity).tags or {}) do
        if tag == "dragon" or tag == "all" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "LOOT_540",
    name = "Dragonhatcher",
    text = "At the end of your turn, <b>Recruit</b> a Dragon.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "epic",
    cost = 9,
    attack = 2,
    health = 4,
    keywords = { "recruit" },
    triggers = {
        {
            event = "turn_ended",
            timing = "after",
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                local candidates = {}
                for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
                    if ctx:entity(entity).type == "minion" and has_dragon_tag(ctx, entity) then
                        candidates[#candidates + 1] = entity
                    end
                end
                if #candidates > 0 then ctx:random_value(candidates, "dragonhatcher_recruit") end
            end,
        },
    },
}

function card.dragonhatcher_recruit(ctx, self, entity)
    ctx:recruit(ctx:controller(self), entity)
end

return card
