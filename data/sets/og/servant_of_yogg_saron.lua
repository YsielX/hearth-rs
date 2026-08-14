local function expensive_spells(ctx)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        local excluded = false
        for _, keyword in ipairs(definition.keywords or {}) do
            if keyword == "quest" or keyword == "questline" or keyword == "cannot_be_randomly_generated" then
                excluded = true
                break
            end
        end
        if definition.type == "spell" and definition.cost >= 5 and not excluded then
            result[#result + 1] = card_id
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "OG_087",
    name = "Servant of Yogg-Saron",
    text = "[x]<b>Battlecry:</b> Cast a random\n spell that costs (5) or MORE \n <i>(targets chosen randomly)</i>.",
    set = "OG",
    type = "minion",
    class = "mage",
    rarity = "rare",
    cost = 5,
    attack = 5,
    health = 4,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        cardlib.random_spell.choose(
            ctx,
            ctx:controller(self),
            expensive_spells(ctx),
            1,
            "servant_spell_chosen"
        )
    end,
}

function card.servant_spell_chosen(ctx, self, choice)
    cardlib.random_spell.cast(ctx, choice)
end

return card
