local function is_murloc(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "murloc" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "GVG_040",
    name = "Siltfin Spiritwalker",
    text = "Whenever another friendly Murloc dies, draw a card. <b><b>Overload</b>:</b> (1)",
    set = "GVG",
    type = "minion",
    class = "shaman",
    rarity = "epic",
    cost = 4,
    attack = 2,
    health = 5,
    tags = { "murloc" },
    keywords = { "overload" },
    keyword_params = { overload = 1 },
    triggers = {
        {
            event = "entity_died",
            timing = "after",
            active_zones = { "board" },
            condition = function(ctx, self, event)
                return event.entity ~= self
                    and event.player == ctx:controller(self)
                    and is_murloc(ctx, event.entity)
            end,
            effect = function(ctx, self)
                ctx:draw(ctx:controller(self), 1)
            end,
        },
    },
}
