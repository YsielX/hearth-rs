local card = {
    api_version = 1,
    id = "LOOT_394",
    name = "Shrieking Shroom",
    text = "At the end of your turn, summon a random\n1-Cost minion.",
    set = "LOOTAPALOOZA",
    type = "minion",
    rarity = "rare",
    cost = 3,
    attack = 1,
    health = 2,
    triggers = {
        {
            event = "turn_ended",
            timing = "after",
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self)
                local pool = {}
                for _, card_id in ipairs(ctx:collectible_cards()) do
                    local definition = ctx:card_definition(card_id)
                    if definition.type == "minion" and definition.cost == 1 then
                        pool[#pool + 1] = card_id
                    end
                end
                if #pool > 0 then ctx:random_value(pool, "shrieking_shroom_minion_chosen") end
            end,
        },
    },
}

function card.shrieking_shroom_minion_chosen(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
end

return card
