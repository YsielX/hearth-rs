local function has_keyword(entity, wanted)
    for _, keyword in ipairs(entity.keywords) do
        if keyword == wanted then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "GDB_100",
    name = "Arkonite Defense Crystal",
    text = "<b>Taunt</b>\n<b>Deathrattle:</b> Gain 4 Armor. <b>Starship Piece</b>",
    set = "SPACE",
    type = "minion",
    rarity = "rare",
    cost = 4,
    attack = 3,
    health = 4,
    keywords = { "taunt", "deathrattle", "starship" },
}

function card.on_deathrattle(ctx, self)
    ctx:gain_armor(ctx:controller(self), 4)
end

function card.on_starship_piece(ctx, self)
    local player = ctx:controller(self)
    for _, entity in ipairs(ctx:hand(player)) do
        if ctx:entity(entity).card_id == "GDB_100t2" then return end
    end
    cardlib.effects.give_card(ctx, player, "GDB_100t2")
end

card.tokens = {
    {
        id = "GDB_100t2",
        name = "The Exile's Hope",
        text = "<b>Starship</b>\n<i>(Costs (5) Mana to launch.)</i>",
        set = "SPACE",
        type = "minion",
        collectible = false,
        cost = 0,
        attack = 0,
        health = 1,
        keywords = { "taunt", "deathrattle" },
        card_actions = {
            launch = {
                active_zones = { "hand" },
                cost = 5,
                condition = function(ctx, self)
                    return ctx:get_data(self, "starship_pieces") > 0
                end,
            },
        },
        action_effects = {
            launch = function(ctx, self)
                ctx:summon_from_hand(self)
            end,
        },
        on_deathrattle = function(ctx, self)
            ctx:gain_armor(ctx:controller(self), ctx:get_data(self, "starship_armor"))
        end,
        triggers = {
            {
                event = "card_created",
                timing = "after",
                active_zones = { "hand" },
                condition = function(ctx, self, event)
                    return event.entity == self
                        and has_keyword(ctx:entity(event.source), "starship")
                end,
                effect = function(ctx, self, event)
                    local piece = ctx:entity(event.source)
                    cardlib.effects.buff(ctx, self, piece.attack, piece.max_health - 1)
                    ctx:set_data(self, "starship_pieces", 1)
                    ctx:set_data(self, "starship_armor", 4)
                end,
            },
            {
                event = "entity_died",
                timing = "after",
                active_zones = { "hand" },
                condition = function(ctx, self, event)
                    return event.player == ctx:controller(self)
                        and has_keyword(ctx:entity(event.entity), "starship")
                end,
                effect = function(ctx, self, event)
                    local piece = ctx:entity(event.entity)
                    cardlib.effects.buff(ctx, self, piece.attack, piece.max_health)
                    ctx:set_data(self, "starship_pieces", ctx:get_data(self, "starship_pieces") + 1)
                    ctx:set_data(self, "starship_armor", ctx:get_data(self, "starship_armor") + 4)
                end,
            },
        },
    },
}

return card
