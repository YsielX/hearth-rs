local card = { api_version = 1, id = "UNG_116", name = "Jungle Giants",
    text = "[x]<b>Quest:</b> Summon\n4 minions with\n5 or more Attack.\n<b>Reward:</b> Barnabus.",
    set = "UNGORO", type = "spell", class = "druid", rarity = "legendary", cost = 1,
    keywords = { "quest" }, triggers = {{ event = "minion_summoned", timing = "after", active_zones = { "secret" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and ctx:get_data(self, "completed") == 0
                and ctx:entity(event.entity).attack >= 5
        end,
        effect = function(ctx, self)
            local progress = ctx:get_data(self, "progress") + 1
            ctx:set_data(self, "progress", progress)
            if progress >= 4 then
                ctx:set_data(self, "completed", 1)
                ctx:reveal_secret(self)
                ctx:give_card(ctx:controller(self), "UNG_116t")
            end
        end }},
}
card.tokens = {{ id = "UNG_116t", name = "Barnabus the Stomper",
    text = "<b>Battlecry:</b> Reduce the\nCost of minions in your deck to (0).", set = "UNGORO",
    type = "minion", class = "druid", cost = 5, attack = 8, health = 8, tags = { "beast" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
            if ctx:entity(entity).type == "minion" then
                cardlib.effects.modify(ctx, entity, { stat = "cost", operation = "set", value = 0, silenciable = false })
            end
        end
    end }}
return card
