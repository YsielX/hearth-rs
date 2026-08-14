local card = {
    api_version = 1,
    id = "DRG_051",
    name = "Strength in Numbers",
    text = "<b>Sidequest:</b> Spend 10 Mana on minions.\n<b>Reward:</b> Summon a minion from your deck.",
    set = "DRAGONS",
    type = "spell",
    rarity = "common",
    class = "druid",
    cost = 1,
    keywords = { "sidequest" },
    triggers = {
        {
            event = "minion_played",
            timing = "after",
            active_zones = { "secret" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
            end,
            effect = function(ctx, self, event)
                local spent = ctx:get_data(self, "mana_spent") + ctx:entity(event.entity).cost
                ctx:set_data(self, "mana_spent", spent)
                if spent < 10 then return end

                local candidates = {}
                for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
                    if ctx:entity(entity).type == "minion" then
                        candidates[#candidates + 1] = entity
                    end
                end
                ctx:reveal_secret(self)
                if #candidates > 0 then
                    ctx:random_entity(candidates, "on_sidequest_recruit")
                end
            end,
        },
    },
}

function card.on_sidequest_recruit(ctx, self, entity)
    ctx:recruit(ctx:controller(self), entity)
end

return card
