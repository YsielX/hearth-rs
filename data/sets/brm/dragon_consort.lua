local ACTIVE_KEY = "dragon_consort_discount"
local PENDING_KEY = "dragon_consort_pending"

local card = {
    api_version = 1,
    id = "BRM_018",
    name = "Dragon Consort",
    text = "<b>Battlecry:</b> The next Dragon you play costs (2) less.",
    set = "BRM",
    type = "minion",
    class = "paladin",
    rarity = "rare",
    cost = 5,
    attack = 5,
    health = 5,
    tags = { "dragon" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local pending = ctx:get_player_data(player, PENDING_KEY)
    ctx:set_player_data(player, PENDING_KEY, pending + 1)
    ctx:grant_player_keyword(player, "dragon_consort_discount")
end

card.triggers = {
    {
        event = "minion_summoned",
        timing = "after",
        active_zones = { "board", "graveyard", "hand", "deck", "removed" },
        condition = function(ctx, self, event)
            return event.entity == self
                and ctx:get_player_data(ctx:controller(self), PENDING_KEY) > 0
        end,
        effect = function(ctx, self)
            local player = ctx:controller(self)
            local pending = ctx:get_player_data(player, PENDING_KEY)
            local active = ctx:get_player_data(player, ACTIVE_KEY)
            ctx:set_player_data(player, ACTIVE_KEY, active + pending)
            ctx:set_player_data(player, PENDING_KEY, 0)
        end,
    },
}

return card
