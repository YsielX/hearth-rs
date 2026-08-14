local card = {
    api_version = 1, id = "AT_002", name = "Effigy",
    text = "<b>Secret:</b> When a friendly minion dies, summon a random minion with the same Cost.",
    set = "TGT", type = "spell", class = "mage", rarity = "rare",
    spell_school = "fire", cost = 3, keywords = { "secret" },
}

card.triggers = {
    {
        event = "entity_died", timing = "after", active_zones = { "secret" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
                and #ctx:board(ctx:controller(self)) < 7
        end,
        effect = function(ctx, self, event)
            ctx:reveal_secret(self)
            ctx:continue_with_value("begin_effigy", {
                cost = ctx:entity(event.entity).cost,
                position = event.position,
            })
        end,
    },
}

function card.begin_effigy(ctx, self, dead)
    if ctx:get_data(self, "triggered") == 1 then return end
    ctx:set_data(self, "triggered", 1)
    ctx:set_data(self, "summon_position", dead.position)
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        if definition.type == "minion" and definition.cost == dead.cost then
            pool[#pool + 1] = id
        end
    end
    if #pool > 0 then ctx:random_value(pool, "summon_effigy") end
end

function card.summon_effigy(ctx, self, card_id)
    ctx:summon_at(ctx:controller(self), card_id, ctx:get_data(self, "summon_position"))
end

return card
