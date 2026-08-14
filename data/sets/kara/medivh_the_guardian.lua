local card = {
    api_version = 1,
    id = "KAR_097",
    name = "Medivh, the Guardian",
    text = "<b>Battlecry:</b> Equip Atiesh, Greatstaff of the Guardian.",
    set = "KARA",
    type = "minion",
    rarity = "legendary",
    cost = 8,
    attack = 7,
    health = 7,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        ctx:equip_weapon(ctx:controller(self), "KAR_097t")
    end,
}

card.tokens = {{
    id = "KAR_097t",
    name = "Atiesh",
    text = "[x]After you cast a spell,\nsummon a random\nminion of that Cost.\nLose 1 Durability.",
    set = "KARA",
    type = "weapon",
    cost = 3,
    attack = 1,
        health = 3,
        triggers = {
            {
                event = "card_played",
                timing = "after",
                active_zones = { "weapon" },
                condition = function(ctx, self, event)
                    return event.player == ctx:controller(self)
                        and ctx:entity(event.entity).type == "spell"
                end,
                effect = function(ctx, self, event)
                    ctx:set_data(self, "played_spell", event.entity)
                    ctx:set_data(self, "played_spell_cost", event.cost)
                end,
        },
        {
            event = "spell_cast",
            timing = "after",
            active_zones = { "weapon" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self) and event.player_cast
            end,
            effect = function(ctx, self, event)
                local cost = ctx:entity(event.entity).cost
                if ctx:get_data(self, "played_spell") == event.entity then
                    cost = ctx:get_data(self, "played_spell_cost")
                end

                local pool = {}
                if #ctx:board(ctx:controller(self)) < 7 then
                    for _, card_id in ipairs(ctx:collectible_cards()) do
                        local definition = ctx:card_definition(card_id)
                        if definition.type == "minion" and definition.cost == cost then
                            pool[#pool + 1] = card_id
                        end
                    end
                end
                if #pool > 0 then
                    ctx:random_value(pool, "summon_minion_then_lose_durability")
                else
                    ctx:lose_weapon_durability(self, 1)
                end
            end,
        },
    },
}}

card.tokens[1].summon_minion_then_lose_durability = function(ctx, self, card_id)
    ctx:summon(ctx:controller(self), card_id)
    ctx:lose_weapon_durability(self, 1)
end

return card
