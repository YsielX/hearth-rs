local card = {
    api_version = 1, id = "OG_090", name = "Cabalist's Tome",
    text = "Get 3 random\nMage spells.", set = "OG", type = "spell",
    class = "mage", rarity = "epic", spell_school = "arcane", cost = 4,
}
local function pool(ctx)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "spell" and definition.class == "mage" then
            result[#result + 1] = card_id
        end
    end
    return result
end
local function choose(ctx, self, hook)
    local candidates = pool(ctx)
    if #candidates > 0 then ctx:random_value(candidates, hook) end
end
function card.on_play(ctx, self) choose(ctx, self, "receive_first_spell") end
function card.receive_first_spell(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id); choose(ctx, self, "receive_second_spell")
end
function card.receive_second_spell(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id); choose(ctx, self, "receive_third_spell")
end
function card.receive_third_spell(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end
return card
