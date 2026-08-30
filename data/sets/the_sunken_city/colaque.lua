local card = {
    api_version = 1,
    id = "TSC_026", rarity = "legendary",
    name = "Colaque",
    text = "[x]<b>Colossal +1</b>\n <b>Immune</b> while you control\nColaque's Shell.",
    set = "THE_SUNKEN_CITY",
    type = "minion",
    class = "druid",
    cost = 7,
    attack = 6,
    health = 5,
    tags = { "beast" },
    keywords = { "colossal" },
    keyword_params = { colossal = 1 },
}

function card.on_colossal(ctx, self)
    cardlib.effects.summon_at(ctx, ctx:controller(self), "TSC_026t", ctx:board_position(self) + 1)
end

card.tokens = {
    {
        id = "TSC_026t",
        name = "Colaque's Shell",
        text = "<b>Taunt</b>\n<b>Deathrattle:</b> Gain 8 Armor.",
        set = "THE_SUNKEN_CITY",
        type = "minion",
        class = "druid",
        cost = 5,
        attack = 0,
        health = 8,
        tags = { "beast" },
        keywords = { "taunt", "deathrattle" },
        on_deathrattle = function(ctx, self)
            ctx:gain_armor(ctx:controller(self), 8)
        end,
        auras = {
            {
                keywords = { "immune" },
                targets = function(ctx, self)
                    local result = {}
                    for _, entity in ipairs(ctx:friendly_minions(self)) do
                        if ctx:entity(entity).card_id == "TSC_026" then
                            result[#result + 1] = entity
                        end
                    end
                    return result
                end,
            },
        },
    },
}

return card
