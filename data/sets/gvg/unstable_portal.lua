local card = {
    api_version = 1,
    id = "GVG_003",
    name = "Unstable Portal",
    text = "Add a random minion to your hand. It costs (3) less.",
    set = "GVG",
    type = "spell",
    class = "mage",
    spell_school = "arcane",
    rarity = "rare",
    cost = 2,
    triggers = {
        {
            event = "card_created",
            timing = "after",
            active_zones = { "graveyard" },
            condition = function(ctx, self, event)
                return event.source == self and event.player == ctx:controller(self)
            end,
            effect = function(ctx, self, event)
                cardlib.effects.modify(ctx, event.entity, { stat = "cost", operation = "add", value = -3 })
            end,
        },
    },
}

function card.on_play(ctx, self)
    local candidates = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        if ctx:card_definition(card_id).type == "minion" then
            candidates[#candidates + 1] = card_id
        end
    end
    if #candidates > 0 then ctx:random_value(candidates, "receive_minion") end
end

function card.receive_minion(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
