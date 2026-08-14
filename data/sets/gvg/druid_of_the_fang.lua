local function is_beast(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "beast" then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "GVG_080",
    name = "Druid of the Fang",
    text = "<b>Battlecry:</b> If you have a Beast, transform this minion into a 7/7.",
    set = "GVG",
    type = "minion",
    class = "druid",
    rarity = "common",
    cost = 5,
    attack = 4,
    health = 4,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self and is_beast(ctx, minion) then
            ctx:transform(self, "GVG_080t")
            return
        end
    end
end

card.tokens = {
    {
        id = "GVG_080t",
        name = "Druid of the Fang",
        text = "",
        set = "GVG",
        type = "minion",
        class = "druid",
        cost = 5,
        attack = 7,
        health = 7,
        tags = { "beast" },
    },
}

return card
