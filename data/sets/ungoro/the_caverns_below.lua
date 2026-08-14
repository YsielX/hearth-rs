local function is_friendly_minion(ctx, self, entity)
    local candidate = ctx:entity(entity)
    return candidate.type == "minion"
        and candidate.controller == ctx:controller(self)
end

local function make_five_five(ctx, self, entity)
    if not is_friendly_minion(ctx, self, entity) then
        return
    end
    ctx:modify_all({ entity }, {
        attack = 5, health = 5, operation = "final_set", silenciable = false, reset_damage = true,
    })
end

local card = {
    api_version = 1,
    id = "UNG_067",
    name = "The Caverns Below",
    text = "[x]<b>Quest:</b> Play four minions\nwith the same name.\n<b>Reward:</b> Crystal Core.",
    set = "UNGORO",
    type = "spell",
    class = "rogue",
    rarity = "legendary",
    cost = 1,
    keywords = { "quest" },

    triggers = {
        {
            event = "minion_played",
            timing = "after",
            active_zones = { "secret" },
            condition = function(ctx, self, event)
                return event.player == ctx:controller(self)
                    and ctx:get_data(self, "completed") == 0
            end,
            effect = function(ctx, self, event)
                local played = ctx:entity(event.entity)
                local name = ctx:card_definition(played.card_id).name
                local key = "played:" .. name
                if #key > 64 then key = "played:" .. played.card_id end
                local count = ctx:get_data(self, key) + 1
                ctx:set_data(self, key, count)
                if count >= 4 then
                    ctx:set_data(self, "completed", 1)
                    ctx:reveal_secret(self)
                    ctx:give_card(ctx:controller(self), "UNG_067t1")
                end
            end,
        },
    },

    tokens = {
        {
            id = "UNG_067t1",
            name = "Crystal Core",
            text = "For the rest of the game, your minions are 5/5.",
            set = "UNGORO",
            type = "spell",
            class = "rogue",
            cost = 5,

            on_play = function(ctx, self)
                local player = ctx:controller(self)
                for _, entity in ipairs(ctx:deck(player)) do make_five_five(ctx, self, entity) end
                for _, entity in ipairs(ctx:hand(player)) do make_five_five(ctx, self, entity) end
                for _, entity in ipairs(ctx:board(player)) do make_five_five(ctx, self, entity) end
            end,

            triggers = {
                {
                    event = "minion_summoned",
                    timing = "after",
                    active_zones = { "graveyard" },
                    condition = function(ctx, self, event)
                        return event.player == ctx:controller(self)
                    end,
                    effect = function(ctx, self, event)
                        make_five_five(ctx, self, event.entity)
                    end,
                },
                {
                    event = "card_created",
                    timing = "after",
                    active_zones = { "graveyard" },
                    condition = function(ctx, self, event)
                        return event.player == ctx:controller(self)
                            and ctx:entity(event.entity).type == "minion"
                    end,
                    effect = function(ctx, self, event)
                        make_five_five(ctx, self, event.entity)
                    end,
                },
                {
                    event = "zone_changed",
                    timing = "after",
                    active_zones = { "graveyard" },
                    condition = function(ctx, self, event)
                        return event.to ~= "graveyard"
                            and is_friendly_minion(ctx, self, event.entity)
                    end,
                    effect = function(ctx, self, event)
                        make_five_five(ctx, self, event.entity)
                    end,
                },
                {
                    event = "transformed",
                    timing = "after",
                    active_zones = { "graveyard" },
                    condition = function(ctx, self, event)
                        return is_friendly_minion(ctx, self, event.entity)
                    end,
                    effect = function(ctx, self, event)
                        make_five_five(ctx, self, event.entity)
                    end,
                },
                {
                    event = "controller_changed",
                    timing = "after",
                    active_zones = { "graveyard" },
                    condition = function(ctx, self, event)
                        return true
                    end,
                    effect = function(ctx, self, event)
                        if event.to == ctx:controller(self) then
                            make_five_five(ctx, self, event.entity)
                        else
                            ctx:remove_enchantments_from(event.entity, self)
                        end
                    end,
                },
            },
        },
    },
}

return card
