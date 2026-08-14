local card = {
    api_version = 1,
    id = "ICC_827",
    name = "Valeera the Hollow",
    text = "<b>Battlecry:</b> Gain <b>Stealth</b> until your next turn.",
    set = "ICECROWN",
    type = "hero",
    class = "rogue",
    cost = 9,
    health = 30,
    armor = 5,
    hero_power = "ICC_827p",
    keywords = { "battlecry" },
    triggers = {
        {
            event = "turn_started", timing = "after", active_zones = { "hero" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and ctx:get_data(self, "veil_of_shadows") == 1
            end,
            effect = function(ctx, self)
                ctx:disable_keyword(self, "stealth")
                ctx:set_data(self, "veil_of_shadows", 0)
            end,
        },
    },
}

function card.on_battlecry(ctx, self)
    ctx:grant_keyword(self, "stealth")
    ctx:set_data(self, "veil_of_shadows", 1)
end

return card
