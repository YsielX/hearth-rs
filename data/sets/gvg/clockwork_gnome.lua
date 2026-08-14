local spare_parts = {
    "PART_001", "PART_002", "PART_003", "PART_004",
    "PART_005", "PART_006", "PART_007",
}

local function all_minions(ctx)
    return ctx:minions()
end

local function friendly_minions(ctx, self)
    return ctx:friendly_minions(self)
end

local card = {
    api_version = 1,
    id = "GVG_082",
    name = "Clockwork Gnome",
    text = "<b>Deathrattle:</b> Add a <b>Spare Part</b> card to your hand.",
    set = "GVG",
    type = "minion",
    rarity = "common",
    cost = 1,
    attack = 2,
    health = 1,
    tags = { "mech" },
    keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    ctx:random_value(spare_parts, "receive_spare_part")
end

function card.receive_spare_part(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

card.tokens = {
    {
        id = "PART_001", name = "Armor Plating", text = "Give a minion +1 Health.",
        set = "GVG", type = "spell", cost = 1, target_mode = "required",
        targets = all_minions,
        on_play = function(ctx, self, target) ctx:buff(target, 0, 1) end,
    },
    {
        id = "PART_002", name = "Time Rewinder", text = "Return a friendly minion to your hand.",
        set = "GVG", type = "spell", cost = 1, target_mode = "required",
        targets = friendly_minions,
        on_play = function(ctx, self, target) ctx:move(target, "hand") end,
    },
    {
        id = "PART_003", name = "Rusty Horn", text = "Give a minion <b>Taunt</b>.",
        set = "GVG", type = "spell", cost = 1, target_mode = "required",
        targets = all_minions,
        on_play = function(ctx, self, target) ctx:grant_keyword(target, "taunt") end,
    },
    {
        id = "PART_004", name = "Finicky Cloakfield",
        text = "Give a friendly minion <b>Stealth</b> until your next turn.",
        set = "GVG", type = "spell", cost = 1, target_mode = "required",
        targets = friendly_minions,
        on_play = function(ctx, self, target)
            ctx:set_data(self, "cloak_target", target)
            ctx:grant_keyword(target, "stealth")
        end,
        triggers = {
            {
                event = "turn_started", active_zones = { "graveyard" },
                condition = function(ctx, self, event)
                    return event.player == ctx:controller(self)
                        and ctx:get_data(self, "cloak_target") ~= 0
                end,
                effect = function(ctx, self)
                    local target = ctx:get_data(self, "cloak_target")
                    ctx:remove_enchantments_from(target, self)
                    ctx:set_data(self, "cloak_target", 0)
                end,
            },
        },
    },
    {
        id = "PART_005", name = "Emergency Coolant", text = "<b>Freeze</b> a minion.",
        set = "GVG", type = "spell", cost = 1, target_mode = "required",
        targets = all_minions,
        on_play = function(ctx, self, target) ctx:freeze(target) end,
    },
    {
        id = "PART_006", name = "Reversing Switch", text = "Swap a minion's Attack and Health.",
        set = "GVG", type = "spell", cost = 1, target_mode = "required",
        targets = all_minions,
        on_play = function(ctx, self, target)
            local minion = ctx:entity(target)
            ctx:modify(target, { stat = "attack", operation = "set", value = minion.health })
            ctx:set_health(target, minion.attack)
        end,
    },
    {
        id = "PART_007", name = "Whirling Blades", text = "Give a minion +1 Attack.",
        set = "GVG", type = "spell", cost = 1, target_mode = "required",
        targets = all_minions,
        on_play = function(ctx, self, target) ctx:buff(target, 1, 0) end,
    },
}

return card
