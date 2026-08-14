return {
    api_version = 1,
    id = "GVG_021",
    name = "Mal'Ganis",
    text = "Your hero is <b>Immune</b>.\nYour other Demons\nhave +2/+2.",
    set = "GVG",
    type = "minion",
    class = "warlock",
    cost = 9,
    attack = 9,
    health = 7,
    tags = { "demon" },
    auras = {
        {
            keywords = { "immune" },
            targets = function(ctx, self)
                return { ctx:player(ctx:controller(self)).hero }
            end,
        },
        {
            attack = 2,
            health = 2,
            targets = function(ctx, self)
                local demons = {}
                for _, minion in ipairs(ctx:friendly_minions(self)) do
                    if minion ~= self then
                        local definition = ctx:card_definition(ctx:entity(minion).card_id)
                        for _, tag in ipairs(definition.tags) do
                            if tag == "demon" then demons[#demons + 1] = minion break end
                        end
                    end
                end
                return demons
            end,
        },
    },
}
