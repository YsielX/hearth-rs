return {
    api_version = 1,
    id = "EX1_287",
    name = "Counterspell",
    text = "<b>Secret:</b> When your opponent casts a spell, <b>Counter</b> it.",
    set = "EXPERT1",
    type = "spell",
    class = "mage",
    cost = 3,
    keywords = { "secret", "counter" },
    triggers = {
        {
            event = "card_played",
            timing = "before",
            active_zones = { "secret" },
            condition = function(ctx, self, event)
                return event.player ~= ctx:controller(self)
                    and ctx:entity(event.entity).type == "spell"
            end,
            effect = function(ctx, self, event)
                ctx:reveal_secret(self)
                ctx:cancel_event(event)
            end,
        },
    },
}
