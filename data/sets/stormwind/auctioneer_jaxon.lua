local card = {
    api_version = 1,
    id = "SW_045",
    name = "Auctioneer Jaxon",
    text = "[x]Whenever you <b>Trade</b>,\n<b>Discover</b> a card from your\n deck to draw instead.",
    set = "STORMWIND",
    type = "minion",
    cost = 2,
    attack = 2,
    health = 3,
}

card.triggers = {
    {
        event = "trade_draw",
        timing = "before",
        active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self)
        end,
        effect = function(ctx, self, event)
            local player = ctx:controller(self)
            ctx:set_data(self, "trade_draw_event", event.event_id)
            ctx:discover_entities(
                player,
                ctx:localize(
                    "Discover a card from your deck to draw",
                    "从你的牌库中发现一张牌来抽取",
                    "從你的牌堆中發現一張牌來抽取"
                ),
                ctx:deck(player),
                3,
                "on_trade_discovered"
            )
        end,
    },
}

card.on_trade_discovered = function(ctx, self, entity)
    ctx:replace_trade_draw(ctx:get_data(self, "trade_draw_event"), entity)
end

return card
