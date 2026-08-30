local function is_undead(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "undead" or tag == "all" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "RLK_121",
    name = "Acolyte of Death",
    text = "After a friendly Undead dies, draw a card.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "minion",
    class = "death_knight",
    rarity = "common",
    cost = 3,
    attack = 2,
    health = 4,
}

card.triggers = {{
    event = "entity_died",
    timing = "after",
    active_zones = { "board" },
    condition = function(ctx, self, event)
        local dead = ctx:entity(event.entity)
        return event.entity ~= self
            and dead.type == "minion"
            and dead.controller == ctx:controller(self)
            and is_undead(ctx, event.entity)
    end,
    effect = function(ctx, self)
        ctx:draw(ctx:controller(self), 1)
    end,
}}

return card
