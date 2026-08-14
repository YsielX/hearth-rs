return {
    api_version = 1,
    id = "CFM_637",
    name = "Patches the Pirate",
    text = "[x]After you play a Pirate,\nsummon this minion\nfrom your deck.",
    set = "GANGS",
    type = "minion",
    cost = 1,
    attack = 1,
    health = 1,
    tags = { "demon", "pirate" },

    triggers = {
        {
            event = "minion_played",
            timing = "after",
            active_zones = { "deck" },
            condition = function(ctx, self, event)
                if event.player ~= ctx:controller(self) then return false end
                local definition = ctx:card_definition(ctx:entity(event.entity).card_id)
                for _, tag in ipairs(definition.tags) do
                    if tag == "pirate" then return true end
                end
                return false
            end,
            effect = function(ctx, self, event)
                ctx:recruit(event.player, self)
            end,
        },
    },
}
